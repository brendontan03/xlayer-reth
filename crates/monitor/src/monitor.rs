use crate::args::FullLinkMonitorArgs;

use alloy_rpc_types_engine::ForkchoiceState;
use futures::StreamExt;
use std::sync::Arc;
use tracing::info;

use alloy_consensus::{transaction::TxHashRef, BlockHeader as _};
use alloy_primitives::B256;
use op_alloy_rpc_types_engine::OpExecutionData;
use reth_node_api::EngineTypes;
use reth_primitives_traits::BlockBody as _;
use reth_provider::CanonStateNotification;

/// Block information for tracing.
#[derive(Debug, Clone)]
pub struct BlockInfo {
    /// The block number
    pub block_number: u64,
    /// The block hash
    pub block_hash: B256,
}

/// XLayerMonitor holds monitoring hook logic for full link monitoring requirements.
#[derive(Clone, Default)]
pub struct XLayerMonitor {
    /// XLayer arguments (reserved for future use)
    #[allow(dead_code)]
    args: FullLinkMonitorArgs,
}

impl XLayerMonitor {
    pub fn new(args: FullLinkMonitorArgs) -> Arc<Self> {
        Arc::new(Self { args })
    }

    /// Handle fork choice updated events, when a FCU API is invoked (before execution).
    pub fn on_fork_choice_updated<EngineT: EngineTypes<ExecutionData = OpExecutionData>>(
        &self,
        _version: &str,
        _state: &ForkchoiceState,
        _attrs: &Option<EngineT::PayloadAttributes>,
    ) {
        // TODO: add SeqBlockBuildStart here
    }

    /// Handle new payload events, when newPayload API is invoked (before execution).
    pub fn on_new_payload(&self, _version: &str, _block_info: &BlockInfo) {
        // TODO: add SeqBlockSendStart, RpcBlockReceiveEnd here, use xlayer_args if you want
    }

    /// Handle transaction received via RPC (eth_sendRawTransaction).
    pub fn on_recv_transaction(&self, _method: &str, _tx_hash: B256) {
        // TODO: add RpcReceiveTxEnd, SeqReceiveTxEnd here, use xlayer_args if you want
    }

    /// Handle block commits to the canonical chain.
    pub fn on_block_commit(&self, _block_info: &BlockInfo) {
        // TODO: add SeqBlockBuildEnd, RpcBlockInsertEnd here
    }

    /// Handle transaction commits to the canonical chain.
    pub fn on_tx_commit(&self, _block_info: &BlockInfo, _tx_hash: B256) {
        // TODO: add SeqTxExecutionEnd here if flashblocks is disabled, you can use xlayer_args if you want
    }

    /// Initialize full link monitor handler for new canonical state changes.
    ///
    /// This spawns a background task that subscribes to canonical state notifications
    /// and calls the monitor's event handlers for each block and transaction.
    pub fn init_canon_state_handle<Node>(self: &Arc<Self>, node: &Node)
    where
        Node: reth_node_api::FullNodeComponents + Clone + 'static,
        <Node as reth_node_api::FullNodeTypes>::Provider: reth_chain_state::CanonStateSubscriptions,
    {
        use reth_chain_state::CanonStateSubscriptions as _;

        let provider = node.provider().clone();
        let task_executor = node.task_executor().clone();

        // Subscribe to canonical state updates
        let canonical_stream = provider.canonical_state_stream();

        let monitor = self.clone();
        task_executor.spawn_critical(
            "xlayer-monitor",
            Box::pin(async move {
                handle_canonical_state_stream(canonical_stream, monitor).await;
            }),
        );
    }
}

/// Handle canonical state stream notifications.
pub async fn handle_canonical_state_stream<N>(
    mut stream: impl StreamExt<Item = CanonStateNotification<N>> + Unpin,
    monitor: Arc<XLayerMonitor>,
) where
    N: reth_primitives_traits::NodePrimitives + 'static,
    N::SignedTx: alloy_consensus::transaction::TxHashRef,
{
    info!(target: "xlayer::monitor", "Blockchain full link monitor started for canon state stream updates");

    while let Some(notification) = stream.next().await {
        match notification {
            CanonStateNotification::Commit { new } => {
                for block in new.blocks_iter() {
                    let sealed_block = block.sealed_block();
                    let block_hash = sealed_block.hash();
                    let block_number = sealed_block.header().number();

                    // Notify block commit
                    let block_info = BlockInfo { block_number, block_hash };
                    monitor.on_block_commit(&block_info);

                    for tx in sealed_block.body().transactions() {
                        // Notify each transaction commit
                        monitor.on_tx_commit(&block_info, *tx.tx_hash());
                    }
                }
            }
            CanonStateNotification::Reorg { new, .. } => {
                for block in new.blocks_iter() {
                    let sealed_block = block.sealed_block();
                    let block_hash = sealed_block.hash();
                    let block_number = sealed_block.header().number();

                    // Notify block commit
                    let block_info = BlockInfo { block_number, block_hash };
                    monitor.on_block_commit(&block_info);

                    for tx in sealed_block.body().transactions() {
                        // Notify transaction commit
                        monitor.on_tx_commit(&block_info, *tx.tx_hash());
                    }
                }
            }
        }
    }

    info!(target: "xlayer::monitor", "Blockchain monitor stopped - canonical state stream closed");
}

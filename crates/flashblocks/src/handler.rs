use op_rbuilder::builders::WebSocketPublisher;
use reth_node_api::FullNodeComponents;
use reth_optimism_flashblocks::FlashBlockRx;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub fn initialize_flashblocks_ws<Node>(
    node: &Node,
    flash_block_rx: FlashBlockRx,
    ws_pub: Arc<WebSocketPublisher>,
) where
    Node: FullNodeComponents,
{
    info!(target: "xlayer::flashblocks-ws", "Initializing flashblocks websocket publisher");

    node.task_executor().spawn_critical(
        "xlayer-flashblocks-ws",
        Box::pin(async move {
            handle_flashblocks_publishing(flash_block_rx, ws_pub).await;
        }),
    );
}

pub async fn handle_flashblocks_publishing(
    mut flash_block_rx: FlashBlockRx,
    ws_pub: Arc<WebSocketPublisher>,
) {
    info!(target: "xlayer::flashblocks-ws", "Flashblocks websocket publisher started");

    loop {
        match flash_block_rx.recv().await {
            Ok(flash_block) => match serde_json::to_string(&*flash_block) {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(payload) => {
                        if let Err(e) = ws_pub.publish(&payload) {
                            warn!(
                                target: "xlayer::flashblocks-ws",
                                "Failed to publish flashblock: {:?}", e
                            );
                        } else {
                            debug!(
                                target: "xlayer::flashblocks-ws",
                                "Published flashblock: index={}, block_hash={}",
                                flash_block.index,
                                flash_block.diff.block_hash
                            );
                        }
                    }
                    Err(e) => {
                        warn!(
                            target: "xlayer::flashblocks-ws",
                            "Failed to deserialize flashblock: {:?}", e
                        );
                    }
                },
                Err(e) => {
                    warn!(
                        target: "xlayer::flashblocks-ws",
                        "Failed to serialize flashblock: {:?}", e
                    );
                }
            },
            Err(e) => {
                warn!(target: "xlayer::flashblocks-ws", "Flashblock receiver error: {:?}", e);
                break;
            }
        }
    }

    info!(target: "xlayer::flashblocks-ws", "Flashblocks websocket publisher stopped");
}

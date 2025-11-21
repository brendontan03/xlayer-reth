//! XLayer Mainnet chain specification

use crate::XLAYER_MAINNET_HARDFORKS;
use alloy_chains::Chain;
use alloy_primitives::{b256, U256};
use once_cell::sync::Lazy;
use reth_chainspec::{BaseFeeParams, BaseFeeParamsKind, ChainSpec, Hardfork};
use reth_ethereum_forks::EthereumHardfork;
use reth_optimism_chainspec::{make_op_genesis_header, OpChainSpec};
use reth_optimism_forks::OpHardfork;
use reth_primitives_traits::SealedHeader;
use std::sync::Arc;

/// XLayer Mainnet genesis hash
const XLAYER_MAINNET_GENESIS_HASH: alloy_primitives::B256 =
    b256!("a5055f0bef5df16f6c69f47a3c2bb0d4a38f618dc1fb6729a10182b64fe34528");

/// XLayer Mainnet chain spec
pub static XLAYER_MAINNET: Lazy<Arc<OpChainSpec>> = Lazy::new(|| {
    let genesis = serde_json::from_str(include_str!("../res/genesis/xlayer-mainnet.json"))
        .expect("Can't deserialize XLayer mainnet genesis json");
    let hardforks = XLAYER_MAINNET_HARDFORKS.clone();
    OpChainSpec {
        inner: ChainSpec {
            chain: Chain::from_id(196),
            genesis_header: SealedHeader::new(
                make_op_genesis_header(&genesis, &hardforks),
                XLAYER_MAINNET_GENESIS_HASH,
            ),
            genesis,
            paris_block_and_final_difficulty: Some((0, U256::from(0))),
            hardforks,
            base_fee_params: BaseFeeParamsKind::Variable(
                vec![
                    (EthereumHardfork::London.boxed(), BaseFeeParams::optimism()),
                    (OpHardfork::Canyon.boxed(), BaseFeeParams::optimism_canyon()),
                ]
                .into(),
            ),
            ..Default::default()
        },
    }
    .into()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xlayer_mainnet_genesis() {
        let spec = &*XLAYER_MAINNET;
        assert_eq!(spec.chain().id(), 196);
    }

    #[test]
    fn test_xlayer_mainnet_is_optimism() {
        use reth_chainspec::EthChainSpec;
        assert!(XLAYER_MAINNET.is_optimism());
    }
}

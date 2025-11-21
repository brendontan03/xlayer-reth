//! XLayer Testnet chain specification

use crate::XLAYER_TESTNET_HARDFORKS;
use alloy_chains::Chain;
use alloy_primitives::{b256, U256};
use once_cell::sync::Lazy;
use reth_chainspec::{BaseFeeParams, BaseFeeParamsKind, ChainSpec, Hardfork};
use reth_ethereum_forks::EthereumHardfork;
use reth_optimism_chainspec::{make_op_genesis_header, OpChainSpec};
use reth_optimism_forks::OpHardfork;
use reth_primitives_traits::SealedHeader;
use std::sync::Arc;

/// XLayer Testnet genesis hash
const XLAYER_TESTNET_GENESIS_HASH: alloy_primitives::B256 =
    b256!("6a745c33ec4110291f2c4d168f3004971d70c5f896f89449001b32c00467436d");

/// XLayer Testnet chain spec
pub static XLAYER_TESTNET: Lazy<Arc<OpChainSpec>> = Lazy::new(|| {
    let genesis = serde_json::from_str(include_str!("../res/genesis/xlayer-testnet.json"))
        .expect("Can't deserialize XLayer testnet genesis json");
    let hardforks = XLAYER_TESTNET_HARDFORKS.clone();
    OpChainSpec {
        inner: ChainSpec {
            chain: Chain::from_id(195),
            genesis_header: SealedHeader::new(
                make_op_genesis_header(&genesis, &hardforks),
                XLAYER_TESTNET_GENESIS_HASH,
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
    fn test_xlayer_testnet_genesis() {
        let spec = &*XLAYER_TESTNET;
        assert_eq!(spec.chain().id(), 195);
    }

    #[test]
    fn test_xlayer_testnet_is_optimism() {
        use reth_chainspec::EthChainSpec;
        assert!(XLAYER_TESTNET.is_optimism());
    }
}

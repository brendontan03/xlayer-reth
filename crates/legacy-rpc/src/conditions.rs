use jsonrpsee::types::Request;

use crate::LegacyRpcRouterService;

impl<S> LegacyRpcRouterService<S> {
    /// Extract block number from request params based on method
    pub(crate) fn extract_block_number(&self, req: &Request<'_>) -> Option<u64> {
        let p = req.params();
        let params = p.as_str()?;
        let method = req.method_name();

        // Parse based on method signature
        // e.g., eth_getBlockByNumber has block as first param
        // eth_getBalance has block as second param
        match method {
            "eth_getBlockByNumber" | "eth_getBlockTransactionCountByNumber" => {
                self.parse_block_param(params, 0)
            }
            "eth_getBalance"
            | "eth_getCode"
            | "eth_getStorageAt"
            | "eth_getTransactionCount"
            | "eth_call" => self.parse_block_param(params, 1),
            _ => None,
        }
    }

    fn parse_block_param(&self, params: &str, index: usize) -> Option<u64> {
        let parsed: serde_json::Value = serde_json::from_str(params).ok()?;
        let arr = parsed.as_array()?;

        // Some params are optional.
        if index >= arr.len() {
            return None;
        }

        let block_param = arr.get(index)?;

        match block_param {
            serde_json::Value::String(s) => {
                // Handle "latest", "pending", "earliest", or hex number
                if s == "latest" || s == "pending" {
                    None // Don't route to legacy
                } else if s == "earliest" {
                    Some(0)
                } else if let Some(hex) = s.strip_prefix("0x") {
                    u64::from_str_radix(hex, 16).ok()
                } else {
                    None
                }
            }
            serde_json::Value::Number(n) => n.as_u64(),
            _ => None,
        }
    }
}

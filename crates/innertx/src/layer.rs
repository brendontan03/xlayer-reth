use tower::Layer;

use crate::InnerTxService;

/// Layer that creates the innertx middleware
#[derive(Clone)]
pub struct InnerTxLayer;

impl Default for InnerTxLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl InnerTxLayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S> Layer<S> for InnerTxLayer {
    type Service = InnerTxService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        InnerTxService { inner }
    }
}

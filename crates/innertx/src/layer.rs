use tower::Layer;

use crate::InnerTxService;

/// Layer that creates the innertx middleware
#[derive(Clone)]
pub struct InnerTxLayer(bool);

impl Default for InnerTxLayer {
    fn default() -> Self {
        Self::new(false)
    }
}

impl InnerTxLayer {
    pub fn new(enabled: bool) -> Self {
        Self(enabled)
    }
}

impl<S> Layer<S> for InnerTxLayer {
    type Service = InnerTxService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        InnerTxService { inner, enabled: self.0 }
    }
}

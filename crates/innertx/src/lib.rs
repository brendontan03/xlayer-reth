#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod layer;

use std::future::Future;

use jsonrpsee::{
    core::middleware::{Batch, Notification},
    server::middleware::rpc::RpcServiceT,
    types::Request,
    MethodResponse,
};
use tracing::debug;

#[derive(Clone)]
pub struct InnerTxService<S> {
    /// how we will forward to next layer
    inner: S,
    /// If disabled, will simply forward to legacy layer
    enabled: bool,
}

impl<S> RpcServiceT for InnerTxService<S>
where
    S: RpcServiceT<MethodResponse = MethodResponse> + Send + Sync + Clone + 'static,
{
    type MethodResponse = MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn call<'a>(&self, req: Request<'a>) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        let method = req.method_name();

        debug!(target:"xlayer_innertx", "calling innertx middleware, method = {}, enabled = {}", method, self.enabled);

        self.inner.call(req)
    }

    fn batch<'a>(&self, req: Batch<'a>) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        // For batches, could implement per-request routing or route entire batch
        self.inner.batch(req)
    }

    fn notification<'a>(
        &self,
        n: Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        self.inner.notification(n)
    }
}

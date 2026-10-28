//! The seam between the tools and the Stride router.
//!
//! The tools are written once, over the router, and they work on both hosts:
//! the desktop and the phone both hold a [`stride_api::Api`] and both answer
//! the same questions. Everything the tools know about Stride goes through
//! the [`StrideApi`] trait, so a host that reaches Stride over a socket
//! instead of in-process can supply its own implementation without touching
//! a single tool.

use std::sync::Arc;

use serde_json::Value;
use stride_api::error::ApiError;
use stride_api::Api;

/// A way to ask Stride a question.
pub trait StrideApi: Send + Sync {
    /// Call a route.
    ///
    /// `method` and `path` are the HTTP method and the path; the query string
    /// may be part of `path`. `body` is the request body, if the route takes
    /// one.
    fn call(&self, method: &str, path: &str, body: Option<&str>) -> Result<Value, ApiError>;
}

impl StrideApi for Api {
    fn call(&self, method: &str, path: &str, body: Option<&str>) -> Result<Value, ApiError> {
        Api::call(self, method, path, body)
    }
}

impl<T: StrideApi + ?Sized> StrideApi for Arc<T> {
    fn call(&self, method: &str, path: &str, body: Option<&str>) -> Result<Value, ApiError> {
        (**self).call(method, path, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_arc_of_the_router_is_a_source() {
        let api: Arc<dyn StrideApi> =
            Arc::new(Api::new(Arc::new(stride_api::mem::MemBackend::new())));
        let health = api.call("GET", "/api/v1/health", None);
        assert!(health.is_ok());
    }
}

use axum::{body::Body, extract::Request, http::HeaderName, response::Response};
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RequestIdLayer {
    trust_header: Option<HeaderName>,
}

impl RequestIdLayer {
    pub fn new(trust_header: Option<String>) -> Self {
        Self {
            trust_header: trust_header.and_then(|h| h.parse().ok()),
        }
    }
}

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService {
            inner,
            trust_header: self.trust_header.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
    trust_header: Option<HeaderName>,
}

impl<S> Service<Request> for RequestIdService<S>
where
    S: Service<Request, Response = Response<Body>> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let req_id = self
            .trust_header
            .as_ref()
            .and_then(|header| req.headers().get(header))
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| ulid::Ulid::new().to_string());

        let span = tracing::info_span!("request", req_id = %req_id);
        let _enter = span.enter();

        let span_clone = span.clone();

        let future = self.inner.call(req);

        Box::pin(async move {
            let _enter = span_clone.enter();
            future.await
        })
    }
}

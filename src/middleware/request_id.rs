use std::pin::Pin;

use axum::{extract::Request, http::HeaderValue, response::Response};
use tower::{Layer, Service};
use uuid::Uuid;

#[derive(Clone)]
pub struct AddRequestIdLayer;

impl<S> Layer<S> for AddRequestIdLayer {
    type Service = AddRequestId<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AddRequestId { inner }
    }
}

#[derive(Clone)]
pub struct AddRequestId<S> {
    inner: S,
}

impl<S> Service<Request> for AddRequestId<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, req: Request) -> Self::Future {
        let request_id = Uuid::new_v4().to_string();
        let future = self.inner.call(req);

        Box::pin(async move {
            let span = tracing::info_span!("request", request_id = %request_id);
            let _guard = span.enter();

            let mut res = future.await?;
            res.headers_mut()
                .insert("x-requeest-id", HeaderValue::from_str(&request_id).unwrap());

            Ok(res)
        })
    }
}

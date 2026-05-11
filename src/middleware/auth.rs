use std::pin::Pin;

use axum::{
    RequestExt,
    extract::Request,
    response::{IntoResponse, Response},
};
use tower::{Layer, Service};

use crate::{auth::extractor::AuthUser, state::AppState};

#[derive(Clone)]
pub struct AuthMiddleware {
    pub state: AppState,
}

impl<S> Layer<S> for AuthMiddleware {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    state: AppState,
}

impl<S> Service<Request> for AuthService<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let state = self.state.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let mut req = req;
            let auth_user = match req.extract_parts_with_state::<AuthUser, _>(&state).await {
                Ok(auth_user) => auth_user,
                Err(err) => return Ok(err.into_response()),
            };

            req.extensions_mut().insert(auth_user);

            inner.call(req).await
        })
    }
}

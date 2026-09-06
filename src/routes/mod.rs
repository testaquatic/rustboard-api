pub mod auth;
pub mod comment;
pub mod me;
pub mod post;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::{
    middleware::require_auth::require_auth,
    routes::{
        auth::{login, signup},
        comment::create_comment,
        me::get_me,
        post::{create_post, get_post, list_posts},
    },
    state::AppState,
};

/// 애플리케이션의 라우터를 설정한다.
pub fn app_router() -> Router<AppState> {
    Router::new()
        .merge(auth_routes())
        .merge(me_routes())
        .nest("/posts", post_routes())
}
/// 인증과 관련된 라우터를 생성한다.
fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
}

/// 애플리케이션의 정보를 얻기 위한 라우터를 생성한다.
fn me_routes() -> Router<AppState> {
    Router::new().route("/me", get(get_me))
}

/// 게시물과 관련된 라우터를 생성한다.
fn post_routes() -> Router<AppState> {
    let public = Router::new()
        .route("/", get(list_posts))
        .route("/{id}", get(get_post));

    let protected = Router::new()
        .route("/", post(create_post))
        .route("/{id}/comments", post(create_comment))
        .route_layer(middleware::from_fn(require_auth));

    public.merge(protected)
}

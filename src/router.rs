use axum::{Router, routing::get};
use utoipa::{
    OpenApi,
    openapi::{Info, OpenApiBuilder},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    const_val::PKG_VERSION,
    routes::{
        comments::{CommentsOpenApi, create_comment, list_comments},
        meta::{HealthOpenApi, VersionOpenApi, health, version},
        posts::{PostsOpenApi, create_post, delete_post, get_post, list_posts, update_post},
    },
    state::AppState,
};

/// axum 라우터
pub fn app_routes(config: &Config) -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/posts", get(list_posts).post(create_post))
        .route(
            "/posts/{id}",
            get(get_post).patch(update_post).delete(delete_post),
        )
        .route(
            "/posts/{post_id}/comments",
            get(list_comments).post(create_comment),
        )
        .merge(openapi_router(config))
}

/// swegger 지원을 위한 라우터
fn openapi_router(config: &Config) -> SwaggerUi {
    let mut openapi = OpenApiBuilder::new()
        .info(Info::new(config.service_name.as_str(), PKG_VERSION))
        .build();
    openapi.merge(HealthOpenApi::openapi());
    openapi.merge(VersionOpenApi::openapi());
    openapi.merge(PostsOpenApi::openapi());
    openapi.merge(CommentsOpenApi::openapi());

    SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi)
}

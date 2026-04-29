use axum::{Router, routing::get};
use utoipa::{
    OpenApi,
    openapi::{Info, OpenApiBuilder},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    const_val::PKG_VERSION,
    handler::{
        meta::{HealthOpenApi, VersionOpenApi, health, version},
        post::{PostsOpenApi, create_post, get_post, list_posts},
    },
    state::AppState,
};

pub fn app_routes(config: &Config) -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/{id}", get(get_post))
        .merge(openapi_router(config))
}

fn openapi_router(config: &Config) -> SwaggerUi {
    let mut openapi = OpenApiBuilder::new()
        .info(Info::new(config.service_name.as_str(), PKG_VERSION))
        .build();
    openapi.merge(HealthOpenApi::openapi());
    openapi.merge(VersionOpenApi::openapi());
    openapi.merge(PostsOpenApi::openapi());

    SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi)
}

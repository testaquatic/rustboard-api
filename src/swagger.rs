use utoipa::{
    OpenApi,
    openapi::{Info, OpenApiBuilder},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    handler::{
        auth::AuthOpenApiDoc, comment::CommentOpenApiDoc, meta::MetaOpenApiDoc,
        post::PostOpenApiDoc, ws::WsOpenApiDoc,
    },
    state::AppState,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_swagger_router(app_state: AppState) -> axum::Router {
    let mut api = OpenApiBuilder::new()
        .info(
            Info::builder()
                .title(app_state.app_info.service_name.as_str())
                .version(VERSION)
                .description(Some(format!("{} swagger", app_state.app_info.service_name)))
                .build(),
        )
        .build();

    api.merge(MetaOpenApiDoc::openapi());
    api.merge(PostOpenApiDoc::openapi());
    api.merge(CommentOpenApiDoc::openapi());
    api.merge(AuthOpenApiDoc::openapi());
    api.merge(WsOpenApiDoc::openapi());

    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api)
        .into()
}

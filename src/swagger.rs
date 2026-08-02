use utoipa::{
    OpenApi,
    openapi::{Info, OpenApiBuilder},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    routes::{comment::CommentOpenApiDoc, meta::MetaOpenApiDoc, post::PostOpenApiDoc},
    state::AppState,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_swagger_router(app_state: AppState) -> axum::Router {
    let mut api = OpenApiBuilder::new()
        .info(
            Info::builder()
                .title(app_state.configuration.service_name.as_str())
                .version(VERSION)
                .description(Some(format!(
                    "{} swagger",
                    app_state.configuration.service_name
                )))
                .build(),
        )
        .build();

    api.merge(MetaOpenApiDoc::openapi());
    api.merge(PostOpenApiDoc::openapi());
    api.merge(CommentOpenApiDoc::openapi());

    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api)
        .into()
}

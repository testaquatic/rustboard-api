use rustboard_api::{startup::start_app, configuration::Settings};

#[tokio::main]
async fn main() {
    // 설정을 불러온다.
    let settings = Settings::build().expect("Failed to build settings");

    // Listener를 생성한다.
    let listener = tokio::net::TcpListener::bind(&settings.app_addr)
        .await
        .expect("Failed to bind to address");

    start_app(settings, listener).await;
}

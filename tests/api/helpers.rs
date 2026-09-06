use rustboard_api::{start_up::start_app, configuration::Settings};
use sqlx::{PgPool, QueryBuilder, migrate};
use tokio::task::JoinHandle;
use uuid::Uuid;

pub struct TestClient {
    pub settings: Settings,
    _server_handle: JoinHandle<()>,
}

impl TestClient {}

pub async fn create_test_app() -> TestClient {
    let mut settings = Settings::build().expect("Failed to build settings");
    settings = set_test_db(settings).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to address");
    let listener_port = listener
        .local_addr()
        .expect("Failed to get listener address")
        .port();

    settings.app_addr = format!("127.0.0.1:{}", listener_port);

    let settings_cloned = settings.clone();
    let server_handle = tokio::spawn(async move { start_app(settings_cloned, listener).await });

    TestClient {
        settings,
        _server_handle: server_handle,
    }
}

async fn set_test_db(mut settings: Settings) -> Settings {
    // DB 주소를 분리한다.
    let slash_position = settings.database_url.rfind("/").unwrap();
    let db_url = &settings.database_url[..slash_position];

    // 테스트용 DB를 생성한다.
    let pool = PgPool::connect(db_url)
        .await
        .expect("Failed to connect to database");
    let db_name = format!("{}", Uuid::new_v4().to_string());

    QueryBuilder::new(format!(r#"CREATE DATABASE "{}";"#, db_name))
        .build()
        .execute(&pool)
        .await
        .expect("Failed to create db");

    // 테스트용 DB를 마이그레이션 한다.
    let db_url = format!("{}/{}", db_url, db_name);
    settings.database_url = db_url;
    let pool = PgPool::connect(&settings.database_url)
        .await
        .expect("Failed to connect to database");

    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    settings
}

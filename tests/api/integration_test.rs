use sqlx::{PgPool, Row};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
async fn postgres_container_starts() {
    // PostgresSQL 컨테이너 시작
    let container = Postgres::default()
        .start()
        .await
        .expect("PostgresSQL 컨테이너 시작 실패");

    // 호스트 포트 가져오기
    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("포트 매핑 실패");

    // 연결 URL 구성
    let database_url = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        host_port
    );

    // 연결 테스트
    let pool = PgPool::connect(&database_url).await.expect("DB 연결 실패");
    let row = sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("쿼리 실패");
    assert_eq!(row.get::<i32, _>(0), 1);
}

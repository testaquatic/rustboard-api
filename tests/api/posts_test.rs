use reqwest::StatusCode;

use crate::helpers::TestClient;

#[tokio::test]
async fn test_list_posts() {
    let client = TestClient::new().await;

    let response = client.get("/posts").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse response");

    assert!(body["posts"].is_array());
    assert!(body["total"].is_number());
    assert!(body["page"].is_number());
}

#[tokio::test]
async fn test_create_post_with_auth() {
    let client = TestClient::new().await;

    // 회원가입
    client
        .post_json(
            "/signup",
            &serde_json::json!({
                "email": "alice@example.com",
                "password": "secret123",
                "display_name": "Alice"
            }),
        )
        .await;

    let login_res = client
        .post_json(
            "/login",
            &serde_json::json!({
                "email": "alice@example.com",
                "password": "secret123",
            }),
        )
        .await;

    let token = login_res
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse response")["token"]
        .as_str()
        .unwrap()
        .to_string();

    let res = client
        .post_json_with_token(
            "/posts",
            &serde_json::json!({
                "title": "Test Post",
                "content": "Hello World",
            }),
            &token,
        )
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let json = res
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse response");

    assert_eq!(json["title"].as_str().unwrap(), "Test Post");
    assert_eq!(json["content"].as_str().unwrap(), "Hello World");
}

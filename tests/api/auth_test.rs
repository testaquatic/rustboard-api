use insta::assert_json_snapshot;
use reqwest::StatusCode;

use crate::helpers::TestClient;

#[tokio::test]
async fn test_signup_success() {
    let client = TestClient::new().await;

    let res = client
        .post_json(
            "/signup",
            &serde_json::json!({
                "email": "newuser@example.com",
                "password": "password123",
                "display_name": "New user"
            }),
        )
        .await;

    assert_eq!(res.status(), StatusCode::CREATED);

    let response_json: serde_json::Value = res.json().await.unwrap();

    assert_json_snapshot!(response_json, {
        ".id" => "[id]",
        ".created_at" => "[timestamp]",
        ".updated_at" => "[timestamp]",
    });
}

#[tokio::test]
async fn test_signup_duplicate_email() {
    let client = TestClient::new().await;

    client
        .post_json(
            "/signup",
            &serde_json::json!({
                "email": "newuser@example.com",
                "password": "password123",
                "display_name": "New user1"
            }),
        )
        .await;

    let res = client
        .post_json(
            "/signup",
            &serde_json::json!({
                "email": "newuser@example.com",
                "password": "password123",
                "display_name": "New user2"
            }),
        )
        .await;

    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let response_json: serde_json::Value = res.json().await.unwrap();
    assert_eq!(response_json["error"], "validation_error");
    assert!(
        response_json["message"]
            .as_str()
            .unwrap()
            .contains("이메일")
    );
}

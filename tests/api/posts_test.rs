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

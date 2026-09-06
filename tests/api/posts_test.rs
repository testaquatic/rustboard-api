use reqwest::StatusCode;

use crate::helpers::create_test_app;

#[tokio::test]
async fn test_list_posts() {
    let app = create_test_app().await;

    let response = reqwest::get(&format!("http://{}/posts", &app.settings.app_addr))
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse response");

    assert!(body["posts"].is_array());
    assert!(body["total"].is_number());
    assert!(body["page"].is_number());
}

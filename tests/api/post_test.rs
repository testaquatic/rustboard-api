use axum::http::StatusCode;
use rustboard_api::domain::posts::Post;
use serde_json::json;
use tower::ServiceExt;

use crate::common::{self, InMemoryPostRepositoryTestExt};

#[tokio::test]
async fn create_post_without_token_returns_401() {
    let ctx = common::InMemoryTestContext::new_in_memory();
    let request = common::post_json(
        "/posts",
        serde_json::json!({"title": "테스트 글", "content": "본문입니다"}),
    );
    let response = ctx.app().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_nonexistent_post_returns_404() {
    let ctx = common::InMemoryTestContext::new_in_memory();
    let response = ctx
        .app()
        .oneshot(common::get("/posts/999999"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn signup_then_login_then_create_post() {
    let ctx = common::InMemoryTestContext::new_in_memory();

    // 로그인
    let token = ctx.signup_and_login().await;

    // 글 작성
    let response = ctx
        .app()
        .oneshot(common::with_token(
            common::post_json(
                "/posts",
                json!({"title": "Alice의 첫 글", "content": "테스트입니다"}),
            ),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn list_returns_empty_then_no_posts() {
    let ctx = common::InMemoryTestContext::new_in_memory();
    let response = ctx.app().oneshot(common::get("/posts")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = common::response_json(response).await;
    assert!(json["posts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_returns_seeded_posts() {
    let ctx = common::InMemoryTestContext::new_in_memory();

    // 사전데이터 주입
    let posts = vec![
        Post {
            id: 1,
            title: "첫 번째".to_string(),
            body: "내용 1".to_string(),
            author_id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Post {
            id: 2,
            title: "두 번째".to_string(),
            body: "내용 2".to_string(),
            author_id: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];
    ctx.post_repo.seed(posts).await;

    let response = ctx.app().oneshot(common::get("/posts")).await.unwrap();
    let json = common::response_json(response).await;

    assert_eq!(json["posts"].as_array().unwrap().len(), 2);
    assert_eq!(json["posts"][0]["title"], "첫 번째");
    assert_eq!(json["posts"][1]["title"], "두 번째");
}

#[tokio::test]
async fn owner_can_delete_own_post() {
    let ctx = common::InMemoryTestContext::new_in_memory();

    // 계정 생성과 로그인
    let token = ctx.signup_and_login().await;

    // 글 작성
    let response = ctx
        .app()
        .oneshot(common::with_token(
            common::post_json(
                "/posts",
                json!({"title": "삭제 테스트", "content": "곧 지워질 글"}),
            ),
            &token,
        ))
        .await
        .unwrap();
    let json = common::response_json(response).await;
    let post_id = json["id"].as_i64().unwrap();

    // 삭제
    let response = ctx
        .app()
        .oneshot(common::with_token(
            common::delete(&format!("/posts/{}", post_id)),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 삭제 확인
    let response = ctx
        .app()
        .oneshot(common::get(&format!("/posts/{}", post_id)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

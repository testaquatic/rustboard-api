use std::hint::black_box;

use rustboard_api::test_utils::test_server::TestServer;

fn create_test_server_with_post(count: usize) -> TestServer {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let test_server = TestServer::new_in_memory().await;
        let token = test_server
            .create_test_token("tester@example.com", "test1234", "Tester")
            .await;
        test_server.create_test_post(&token, count).await;

        test_server
    })
}

fn bench_list_post(c: &mut criterion::Criterion) {
    let test_server = create_test_server_with_post(10000);

    c.bench_function("list_posts_1000", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(
                test_server
                    .state
                    .posts_service
                    .list_recent(None, black_box(20)),
            )
        });
    });
}

fn bench_cursor_pagination(c: &mut criterion::Criterion) {
    let test_server = create_test_server_with_post(10000);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let first_page = rt
        .block_on(test_server.state.posts_service.list_recent(None, 20))
        .unwrap();
    let cursor = first_page.last().map(|p| (p.created_at, p.id)).unwrap();

    c.bench_function("cursor_page_10k", |b| {
        b.iter(|| {
            rt.block_on(
                test_server
                    .state
                    .posts_service
                    .list_recent(Some(cursor), black_box(20)),
            )
        });
    });
}

fn bench_pagination_comparison(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("pagination");
    let test_server = create_test_server_with_post(10000);
    let rt = tokio::runtime::Runtime::new().unwrap();

    for size in [100, 1000, 10000] {
        group.bench_with_input(
            criterion::BenchmarkId::new("cursor", size),
            &size,
            |b, &_size| {
                b.iter(|| rt.block_on(test_server.state.posts_service.list_recent(None, 20)));
            },
        );
    }
    group.finish();
}

criterion::criterion_group!(
    benches,
    bench_list_post,
    bench_cursor_pagination,
    bench_pagination_comparison
);
criterion::criterion_main!(benches);

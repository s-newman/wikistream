use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::PgPool;
use tower::ServiceExt;
use ws_app::get_test_router;

#[sqlx::test]
async fn test_health(db: PgPool) {
    let router = get_test_router(db);
    let req = Request::get("/health").body(Body::empty()).unwrap();

    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body, r#"{"status":"healthy"}"#.as_bytes());
}

#[sqlx::test]
async fn test_daily_redirect(db: PgPool) {
    let router = get_test_router(db);
    let req = Request::get("/daily?date=2026-05-01")
        .body(Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);

    let location = resp.headers().get("location").unwrap();
    assert_eq!(location, "/daily/2026-05-01");
}

use axum::Router;
use sqlx::SqlitePool;
use crate::api::{build_router, AppState};
use tower_http::services::ServeDir;

pub async fn run(addr: &str, db_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = crate::db::init_pool(db_url).await?;
    crate::db::migrate(&pool).await?;
    let state = AppState { pool };
    let api = build_router(state);
    let static_service = ServeDir::new("web");
    let app: Router = api.nest_service("/", static_service);
    axum::Server::bind(&addr.parse()?).serve(app.into_make_service()).await?;
    Ok(())
}
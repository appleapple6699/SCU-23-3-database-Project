use database::server::run;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let db_url = "sqlite://data/app.db";
    let _ = run(addr, db_url).await;
}
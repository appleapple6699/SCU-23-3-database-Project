use database::server::run;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let db_path = "data/app.db";
    let _ = std::fs::create_dir_all("data");
    let _ = std::fs::OpenOptions::new().create(true).write(true).open(db_path);
    let db_url = format!("sqlite://{}", db_path);
    if let Err(e) = run(addr, &db_url).await {
        eprintln!("server error: {}", e);
        std::process::exit(1);
    }
}
use bcrypt::{hash, DEFAULT_COST};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let nickname = args.get(1).cloned().unwrap_or_else(|| "admin".to_string());
    let password = args.get(2).cloned().unwrap_or_else(|| "admin123".to_string());
    let db_url = std::env::var("DB_URL").unwrap_or_else(|_| "sqlite://data/app.db".to_string());

    let pool = SqlitePool::connect(&db_url).await.unwrap();
    let _ = database::db::migrate(&pool).await;
    let hashed = hash(password, DEFAULT_COST).unwrap();

    let existing = sqlx::query_scalar::<_, i64>("SELECT UserID FROM Users WHERE Nickname=?1")
        .bind(&nickname)
        .fetch_optional(&pool)
        .await
        .unwrap();

    if let Some(uid) = existing {
        let _ = sqlx::query("UPDATE Users SET Password=?1, IsAdmin=1, IsActive=1 WHERE UserID=?2")
            .bind(hashed)
            .bind(uid)
            .execute(&pool)
            .await;
        println!("updated admin user {} (id={})", nickname, uid);
    } else {
        let res = sqlx::query("INSERT INTO Users (Password, Nickname, IsActive, IsAdmin) VALUES (?1, ?2, 1, 1)")
            .bind(hashed)
            .bind(&nickname)
            .execute(&pool)
            .await
            .unwrap();
        let uid = res.last_insert_rowid();
        println!("created admin user {} (id={})", nickname, uid);
    }
}
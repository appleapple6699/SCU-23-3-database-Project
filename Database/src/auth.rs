use axum::http::HeaderMap;
use sqlx::{Row, SqlitePool};

fn extract_token(headers: &HeaderMap) -> Option<String> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    if let Some(rest) = auth.strip_prefix("Bearer ") { Some(rest.to_string()) } else { None }
}

pub async fn auth_user(pool: &SqlitePool, headers: &HeaderMap) -> Option<(i64, bool)> {
    let token = extract_token(headers)?;
    let row = sqlx::query(
        "SELECT u.UserID, IFNULL(u.IsAdmin,0) as IsAdmin
         FROM Sessions s JOIN Users u ON s.UserID = u.UserID
         WHERE s.Token = ?1 AND (s.Expires IS NULL OR s.Expires > DATETIME('now'))"
    ).bind(token).fetch_optional(pool).await.ok()?;
    row.map(|r| {
        let uid: i64 = r.get(0);
        let is_admin: i64 = r.get(1);
        (uid, is_admin == 1)
    })
}

pub async fn require_admin(pool: &SqlitePool, headers: &HeaderMap) -> Option<i64> {
    match auth_user(pool, headers).await { Some((uid, true)) => Some(uid), _ => None }
}

pub async fn require_leader(pool: &SqlitePool, headers: &HeaderMap, group_id: i64) -> Option<i64> {
    let (uid, _is_admin) = auth_user(pool, headers).await?;
    let ok = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM UserGroups WHERE UserID=?1 AND GroupID=?2 AND GroupPermission=2"
    ).bind(uid).bind(group_id).fetch_one(pool).await.ok()?;
    if ok > 0 { Some(uid) } else { None }
}
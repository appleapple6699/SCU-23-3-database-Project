use axum::{Router, routing::{post, get, patch, delete, put}, extract::{State, Path, Query}, Json};
use serde_json::json;
use sqlx::{SqlitePool, Row};
use crate::models::{UnifiedResponse, RegisterRequest, LoginRequest, UpdateUserRequest, LoginResponse};
use crate::auth;
use axum::http::HeaderMap;
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/users/register", post(register))
        .route("/api/users/login", post(login))
        .route("/api/auth/me", get(auth_me))
        .route("/api/logout", post(logout))
        .route("/api/users/:id", patch(update_user))
        .route("/api/users/:id/freeze", patch(freeze_user))
        .route("/api/groups", post(create_group))
        .route("/api/groups", get(list_groups))
        .route("/api/groups/:id/status", patch(update_group_status))
        .route("/api/groups/:id/disband", delete(disband_group))
        .route("/api/groups/:id/transfer", post(transfer_leader))
        .route("/api/groups/:id/stats", get(group_stats))
        .route("/api/tasks", post(create_task))
        .route("/api/tasks", get(list_tasks))
        .route("/api/tasks/:id", put(update_task))
        .route("/api/tasks/:id", delete(delete_task))
        .route("/api/usergroups/apply", post(apply_usergroup))
        .route("/api/usergroups/applications", get(list_applications))
        .route("/api/usergroups/update", patch(update_usergroup))
        .route("/api/entries", post(create_entry))
        .route("/api/auditentries", post(create_auditentry))
        .route("/api/notifications", post(create_notification))
        .route("/api/notificationconfirmations", post(confirm_notification))
        .route("/api/search", get(search))
        .with_state(state)
}

async fn register(State(state): State<AppState>, Json(req): Json<RegisterRequest>) -> Json<UnifiedResponse<serde_json::Value>> {
    let hashed = match hash(req.password, DEFAULT_COST) { Ok(h) => h, Err(_) => return Json(UnifiedResponse::err(2001, "加密失败")) };
    let res = sqlx::query("INSERT INTO Users (Password, Nickname, IsActive) VALUES (?1, ?2, 1)")
        .bind(hashed)
        .bind(req.nickname)
        .execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2002, "注册失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> Json<UnifiedResponse<LoginResponse>> {
    let row = sqlx::query("SELECT UserID, Password, IsActive FROM Users WHERE Nickname = ?1 OR UserID = ?2")
        .bind(&req.phone)
        .bind(req.phone.parse::<i64>().unwrap_or(-1))
        .fetch_optional(&state.pool).await;
    if let Ok(Some(r)) = row {
        let uid: i64 = r.get(0);
        let hashed: String = r.get(1);
        let active: i64 = r.get(2);
        if active == 0 { return Json(UnifiedResponse::err(2003, "账号已冻结")); }
        if verify(&req.password, &hashed).unwrap_or(false) {
            let token = Uuid::new_v4().to_string();
            let _ = sqlx::query("INSERT INTO Sessions (Token, UserID) VALUES (?1, ?2)").bind(&token).bind(uid).execute(&state.pool).await;
            return Json(UnifiedResponse::ok(LoginResponse { token, user_id: uid }));
        }
    }
    Json(UnifiedResponse::err(2002, "登录失败"))
}

async fn update_user(State(state): State<AppState>, Path(id): Path<i64>, Json(req): Json<UpdateUserRequest>) -> Json<UnifiedResponse<serde_json::Value>> {
    if let Some(nick) = req.nickname.clone() {
        if sqlx::query("UPDATE Users SET Nickname=?1 WHERE UserID=?2").bind(nick).bind(id).execute(&state.pool).await.is_err() {
            return Json(UnifiedResponse::err(2004, "更新失败"));
        }
    }
    if let Some(pw) = req.password.clone() {
        let hashed = match hash(pw, DEFAULT_COST) { Ok(h) => h, Err(_) => { return Json(UnifiedResponse::err(2001, "加密失败")); } };
        if sqlx::query("UPDATE Users SET Password=?1 WHERE UserID=?2").bind(hashed).bind(id).execute(&state.pool).await.is_err() {
            return Json(UnifiedResponse::err(2004, "更新失败"));
        }
    }
    Json(UnifiedResponse::ok(json!({})))
}

async fn freeze_user(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i64>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    if auth::require_admin(&state.pool, &headers).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    let is_active = payload.get("isActive").and_then(|v| v.as_i64()).unwrap_or(1);
    let unfreeze = payload.get("unfreezeDateTime").and_then(|v| v.as_str());
    let res = sqlx::query("UPDATE Users SET IsActive=?1, UnfreezeDateTime=?2 WHERE UserID=?3")
        .bind(is_active)
        .bind(unfreeze)
        .bind(id)
        .execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2010, "冻结/解冻失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn create_group(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let desc = payload.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let creator = payload.get("creatorUserId").and_then(|v| v.as_i64()).unwrap_or(0);
    let res = sqlx::query("INSERT INTO Groups (GroupName, Description, Status, CreatedByUserID) VALUES (?1, ?2, 0, ?3)")
        .bind(name)
        .bind(desc)
        .bind(creator)
        .execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2005, "创建组失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn search(State(state): State<AppState>) -> Json<UnifiedResponse<serde_json::Value>> {
    Json(UnifiedResponse::ok(json!({ "results": [] })))
}

async fn list_groups(State(state): State<AppState>) -> Json<UnifiedResponse<serde_json::Value>> {
    let rows = sqlx::query("SELECT GroupID, GroupName, Description, Status FROM Groups")
        .fetch_all(&state.pool).await;
    if let Ok(rs) = rows {
        let data: Vec<serde_json::Value> = rs.into_iter().map(|r| {
            json!({
                "GroupID": r.get::<i64,_>(0),
                "GroupName": r.get::<String,_>(1),
                "Description": r.get::<String,_>(2),
                "Status": r.get::<i64,_>(3)
            })
        }).collect();
        return Json(UnifiedResponse::ok(json!({"groups": data})));
    }
    Json(UnifiedResponse::err(2006, "查询组失败"))
}

async fn create_task(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    let group_id = payload.get("groupId").and_then(|v| v.as_i64()).unwrap_or(0);
    let publisher_id = payload.get("publisherId").and_then(|v| v.as_i64()).unwrap_or(0);
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let deadline = payload.get("deadline").and_then(|v| v.as_str());
    let mut q = sqlx::query("INSERT INTO Tasks (GroupID, PublisherID, Title, Content, Deadline) VALUES (?1, ?2, ?3, ?4, ?5)")
        .bind(group_id)
        .bind(publisher_id)
        .bind(title)
        .bind(content)
        .bind(deadline);
    if q.execute(&state.pool).await.is_err() { return Json(UnifiedResponse::err(2007, "创建任务失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn list_tasks(State(state): State<AppState>) -> Json<UnifiedResponse<serde_json::Value>> {
    let rows = sqlx::query("SELECT TaskID, GroupID, PublisherID, Title, Deadline, IsValid FROM Tasks")
        .fetch_all(&state.pool).await;
    if let Ok(rs) = rows {
        let data: Vec<serde_json::Value> = rs.into_iter().map(|r| {
            json!({
                "TaskID": r.get::<i64,_>(0),
                "GroupID": r.get::<i64,_>(1),
                "PublisherID": r.get::<i64,_>(2),
                "Title": r.get::<String,_>(3),
                "Deadline": r.get::<String,_>(4),
                "IsValid": r.get::<i64,_>(5)
            })
        }).collect();
        return Json(UnifiedResponse::ok(json!({"tasks": data})));
    }
    Json(UnifiedResponse::err(2008, "查询任务失败"))
}

async fn apply_usergroup(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    let user_id = payload.get("userId").and_then(|v| v.as_i64()).unwrap_or(0);
    let group_id = payload.get("groupId").and_then(|v| v.as_i64()).unwrap_or(0);
    let res = sqlx::query("INSERT INTO UserGroups (UserID, GroupID, GroupPermission, Status) VALUES (?1, ?2, 0, 0)")
        .bind(user_id)
        .bind(group_id)
        .execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2009, "申请失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn list_applications(State(state): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>) -> Json<UnifiedResponse<serde_json::Value>> {
    let group_id = params.get("groupId").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    let rows = sqlx::query("SELECT UserGroupID, UserID, GroupID, Status FROM UserGroups WHERE GroupID=?1 AND Status=0")
        .bind(group_id)
        .fetch_all(&state.pool).await;
    if let Ok(rs) = rows {
        let data: Vec<serde_json::Value> = rs.into_iter().map(|r| {
            json!({
                "UserGroupID": r.get::<i64,_>(0),
                "UserID": r.get::<i64,_>(1),
                "GroupID": r.get::<i64,_>(2),
                "Status": r.get::<i64,_>(3)
            })
        }).collect();
        return Json(UnifiedResponse::ok(json!({"applications": data})));
    }
    Json(UnifiedResponse::err(2011, "查询申请失败"))
}

async fn update_usergroup(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    let user_id = payload.get("userId").and_then(|v| v.as_i64()).unwrap_or(0);
    let group_id = payload.get("groupId").and_then(|v| v.as_i64()).unwrap_or(0);
    let action = payload.get("action").and_then(|v| v.as_i64()).unwrap_or(0);
    let status = if action == 1 { 1 } else if action == 2 { 2 } else { 0 };
    let res = sqlx::query("UPDATE UserGroups SET Status=?1 WHERE UserID=?2 AND GroupID=?3")
        .bind(status)
        .bind(user_id)
        .bind(group_id)
        .execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2012, "更新申请失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn update_group_status(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i64>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    if auth::require_admin(&state.pool, &headers).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    let status = payload.get("status").and_then(|v| v.as_i64()).unwrap_or(0);
    let res = sqlx::query("UPDATE Groups SET Status=?1 WHERE GroupID=?2").bind(status).bind(id).execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2013, "更新组状态失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn delete_task(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i64>) -> Json<UnifiedResponse<serde_json::Value>> {
    // 仅管理员或具有任务所属组的组长可删除（示例简单化：管理员）
    if auth::require_admin(&state.pool, &headers).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    let res = sqlx::query("DELETE FROM Tasks WHERE TaskID=?1").bind(id).execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2014, "删除任务失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn update_task(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i64>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    if auth::require_admin(&state.pool, &headers).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    let title = payload.get("title").and_then(|v| v.as_str());
    let content = payload.get("content").and_then(|v| v.as_str());
    let deadline = payload.get("deadline").and_then(|v| v.as_str());
    let res = sqlx::query("UPDATE Tasks SET Title=COALESCE(?1, Title), Content=COALESCE(?2, Content), Deadline=COALESCE(?3, Deadline) WHERE TaskID=?4")
        .bind(title)
        .bind(content)
        .bind(deadline)
        .bind(id)
        .execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2015, "更新任务失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn disband_group(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i64>) -> Json<UnifiedResponse<serde_json::Value>> {
    if auth::require_admin(&state.pool, &headers).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    if sqlx::query("UPDATE Groups SET Status=2 WHERE GroupID=?1").bind(id).execute(&state.pool).await.is_err() {
        return Json(UnifiedResponse::err(2016, "解散组失败"));
    }
    if sqlx::query("DELETE FROM UserGroups WHERE GroupID=?1").bind(id).execute(&state.pool).await.is_err() {
        return Json(UnifiedResponse::err(2016, "解散组失败"));
    }
    Json(UnifiedResponse::ok(json!({})))
}

async fn transfer_leader(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i64>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    if auth::require_leader(&state.pool, &headers, id).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    let from_id = payload.get("fromUserId").and_then(|v| v.as_i64()).unwrap_or(0);
    let to_id = payload.get("toUserId").and_then(|v| v.as_i64()).unwrap_or(0);
    if sqlx::query("UPDATE UserGroups SET GroupPermission=0 WHERE UserID=?1 AND GroupID=?2")
        .bind(from_id).bind(id).execute(&state.pool).await.is_err() { return Json(UnifiedResponse::err(2017, "转让失败")); }
    if sqlx::query("UPDATE UserGroups SET GroupPermission=2 WHERE UserID=?1 AND GroupID=?2")
        .bind(to_id).bind(id).execute(&state.pool).await.is_err() { return Json(UnifiedResponse::err(2017, "转让失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn group_stats(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<i64>) -> Json<UnifiedResponse<serde_json::Value>> {
    if auth::require_leader(&state.pool, &headers, id).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    let tasks = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM Tasks WHERE GroupID=?1").bind(id).fetch_one(&state.pool).await.unwrap_or(0);
    let entries = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM Entries e JOIN Tasks t ON e.TaskID=t.TaskID WHERE t.GroupID=?1").bind(id).fetch_one(&state.pool).await.unwrap_or(0);
    let confirms = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM NotificationConfirmations nc JOIN Notifications n ON nc.NotificationID=n.NotificationID WHERE n.GroupID=?1").bind(id).fetch_one(&state.pool).await.unwrap_or(0);
    Json(UnifiedResponse::ok(json!({"tasks": tasks, "entries": entries, "confirmations": confirms})))
}

async fn create_entry(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    let task_id = payload.get("taskId").and_then(|v| v.as_i64()).unwrap_or(0);
    let submitter_id = payload.get("submitterId").and_then(|v| v.as_i64()).unwrap_or(0);
    let summary = payload.get("summary").and_then(|v| v.as_str()).unwrap_or("");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let res = sqlx::query("INSERT INTO Entries (TaskID, SubmitterID, Summary, Content) VALUES (?1, ?2, ?3, ?4)")
        .bind(task_id).bind(submitter_id).bind(summary).bind(content).execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2018, "提交条目失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn create_auditentry(State(state): State<AppState>, headers: HeaderMap, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    if auth::auth_user(&state.pool, &headers).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    let entry_id = payload.get("entryId").and_then(|v| v.as_i64()).unwrap_or(0);
    let auditor_id = payload.get("auditorId").and_then(|v| v.as_i64()).unwrap_or(0);
    let result = payload.get("auditResult").and_then(|v| v.as_i64()).unwrap_or(0);
    let desc = payload.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let res = sqlx::query("INSERT INTO AuditEntries (EntryID, AuditorID, AuditResult, Description) VALUES (?1, ?2, ?3, ?4)")
        .bind(entry_id).bind(auditor_id).bind(result).bind(desc).execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2019, "审核失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn create_notification(State(state): State<AppState>, headers: HeaderMap, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    let group_id = payload.get("groupId").and_then(|v| v.as_i64()).unwrap_or(0);
    if group_id == 0 {
        if auth::require_admin(&state.pool, &headers).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    } else {
        if auth::require_leader(&state.pool, &headers, group_id).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    }
    let publisher_id = payload.get("publisherId").and_then(|v| v.as_i64()).unwrap_or(0);
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let res = sqlx::query("INSERT INTO Notifications (PublisherID, GroupID, Title, Content) VALUES (?1, ?2, ?3, ?4)")
        .bind(publisher_id).bind(group_id).bind(title).bind(content).execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2020, "发布通知失败")); }
    Json(UnifiedResponse::ok(json!({})))
}

async fn confirm_notification(State(state): State<AppState>, headers: HeaderMap, Json(payload): Json<serde_json::Value>) -> Json<UnifiedResponse<serde_json::Value>> {
    if auth::auth_user(&state.pool, &headers).await.is_none() { return Json(UnifiedResponse::err(401, "未授权")); }
    let notification_id = payload.get("notificationId").and_then(|v| v.as_i64()).unwrap_or(0);
    let user_id = payload.get("userId").and_then(|v| v.as_i64()).unwrap_or(0);
    let res = sqlx::query("INSERT INTO NotificationConfirmations (NotificationID, UserID) VALUES (?1, ?2)")
        .bind(notification_id).bind(user_id).execute(&state.pool).await;
    if res.is_err() { return Json(UnifiedResponse::err(2021, "确认通知失败")); }
    Json(UnifiedResponse::ok(json!({})))
}
async fn auth_me(State(state): State<AppState>, headers: HeaderMap) -> Json<UnifiedResponse<serde_json::Value>> {
    if let Some((uid, is_admin)) = auth::auth_user(&state.pool, &headers).await {
        let row = sqlx::query("SELECT Nickname FROM Users WHERE UserID=?1").bind(uid).fetch_one(&state.pool).await;
        if let Ok(r) = row {
            let nick: String = r.get(0);
            return Json(UnifiedResponse::ok(json!({"userId": uid, "nickname": nick, "isAdmin": is_admin})));
        }
    }
    Json(UnifiedResponse::err(401, "未登录"))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Json<UnifiedResponse<serde_json::Value>> {
    if let Some(token) = headers.get("authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")).map(|s| s.to_string()) {
        let _ = sqlx::query("DELETE FROM Sessions WHERE Token=?1").bind(token).execute(&state.pool).await;
        return Json(UnifiedResponse::ok(json!({})))
    }
    Json(UnifiedResponse::err(401, "未登录"))
}
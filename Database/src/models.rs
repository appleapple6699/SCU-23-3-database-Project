use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UnifiedResponse<T> {
    pub error_code: i32,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> UnifiedResponse<T> {
    pub fn ok(data: T) -> Self { Self { error_code: 0, msg: String::from(""), data: Some(data) } }
    pub fn err(code: i32, msg: &str) -> Self { Self { error_code: code, msg: msg.to_string(), data: None } }
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub nickname: String,
    pub password: String,
    pub phone: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub phone: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub nickname: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: i64,
}
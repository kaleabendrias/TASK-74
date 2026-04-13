use gloo_net::http::Request;

use crate::auth;
use crate::models::{LoginRequest, LoginResponse};

const BASE_URL: &str = "/api";

pub async fn login(username: &str, password: &str) -> Result<LoginResponse, String> {
    let body = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
        totp_code: None,
    };

    let resp = Request::post(&format!("{}/login", BASE_URL))
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<LoginResponse>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(format!("Login failed (status {})", resp.status()))
    }
}

pub async fn health() -> Result<crate::models::HealthResponse, String> {
    let mut req = Request::get(&format!("{}/health", BASE_URL));

    if let Some(token) = auth::get_session_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Health check failed (status {})", resp.status()))
    }
}

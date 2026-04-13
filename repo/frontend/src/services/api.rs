//! HTTP API client with CSRF token injection via X-CSRF-Token header.
//! Provides typed async functions for every backend endpoint.

use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::models::*;

const BASE: &str = "/api";

fn csrf_token() -> Option<String> {
    // Read from the AuthProvider context via a thread-local
    CSRF_TOKEN.with(|t| t.borrow().clone())
}

std::thread_local! {
    pub static CSRF_TOKEN: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

pub fn set_csrf_token(token: Option<String>) {
    CSRF_TOKEN.with(|t| *t.borrow_mut() = token);
}

async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let mut req = Request::get(url);
    if let Some(token) = csrf_token() {
        req = req.header("X-CSRF-Token", &token);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let err = resp.json::<ApiError>().await.unwrap_or(ApiError {
            code: "UNKNOWN".into(),
            message: format!("HTTP {}", resp.status()),
            details: vec![],
        });
        Err(err.message)
    }
}

async fn post_json<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, String> {
    let mut req = Request::post(url);
    if let Some(token) = csrf_token() {
        req = req.header("X-CSRF-Token", &token);
    }
    let resp = req.json(body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() || resp.status() == 201 {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let err = resp.json::<ApiError>().await.unwrap_or(ApiError {
            code: "UNKNOWN".into(),
            message: format!("HTTP {}", resp.status()),
            details: vec![],
        });
        Err(err.message)
    }
}

async fn put_json<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, String> {
    let mut req = Request::put(url);
    if let Some(token) = csrf_token() {
        req = req.header("X-CSRF-Token", &token);
    }
    let resp = req.json(body).map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() || resp.status() == 201 {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let err = resp.json::<ApiError>().await.unwrap_or(ApiError {
            code: "UNKNOWN".into(),
            message: format!("HTTP {}", resp.status()),
            details: vec![],
        });
        Err(err.message)
    }
}

async fn post_empty<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let mut req = Request::post(url);
    if let Some(token) = csrf_token() {
        req = req.header("X-CSRF-Token", &token);
    }
    let resp = req.header("Content-Type", "application/json")
        .body("{}").map_err(|e| e.to_string())?
        .send().await.map_err(|e| e.to_string())?;
    if resp.ok() || resp.status() == 201 {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let err = resp.json::<ApiError>().await.unwrap_or(ApiError {
            code: "UNKNOWN".into(),
            message: format!("HTTP {}", resp.status()),
            details: vec![],
        });
        Err(err.message)
    }
}

// ── Auth ──
pub async fn login(req: &LoginRequest) -> Result<LoginResponse, String> {
    post_json(&format!("{}/auth/login", BASE), req).await
}

pub async fn logout() -> Result<serde_json::Value, String> {
    post_empty(&format!("{}/auth/logout", BASE)).await
}

pub async fn me() -> Result<UserProfile, String> {
    get_json(&format!("{}/auth/me", BASE)).await
}

// ── Resources ──
pub async fn list_resources(
    page: i64, per_page: i64, state: &str, category: &str, search: &str, sort_by: &str
) -> Result<PaginatedResponse<ResourceResponse>, String> {
    let mut url = format!("{}/resources?page={}&per_page={}", BASE, page, per_page);
    if !state.is_empty() { url.push_str(&format!("&state={}", state)); }
    if !category.is_empty() { url.push_str(&format!("&category={}", category)); }
    if !search.is_empty() { url.push_str(&format!("&search={}", search)); }
    if !sort_by.is_empty() { url.push_str(&format!("&sort_by={}", sort_by)); }
    get_json(&url).await
}

pub async fn get_resource(id: &str) -> Result<ResourceResponse, String> {
    get_json(&format!("{}/resources/{}", BASE, id)).await
}

pub async fn create_resource(req: &CreateResourceRequest) -> Result<ResourceResponse, String> {
    post_json(&format!("{}/resources", BASE), req).await
}

pub async fn update_resource(id: &str, req: &UpdateResourceRequest) -> Result<ResourceResponse, String> {
    put_json(&format!("{}/resources/{}", BASE, id), req).await
}

// ── Lodgings ──
pub async fn list_lodgings() -> Result<Vec<LodgingResponse>, String> {
    get_json(&format!("{}/lodgings", BASE)).await
}

pub async fn get_lodging(id: &str) -> Result<LodgingResponse, String> {
    get_json(&format!("{}/lodgings/{}", BASE, id)).await
}

pub async fn create_lodging(req: &CreateLodgingRequest) -> Result<LodgingResponse, String> {
    post_json(&format!("{}/lodgings", BASE), req).await
}

pub async fn update_lodging(id: &str, req: &UpdateLodgingRequest) -> Result<LodgingResponse, String> {
    put_json(&format!("{}/lodgings/{}", BASE, id), req).await
}

pub async fn get_periods(lodging_id: &str) -> Result<Vec<LodgingPeriodResponse>, String> {
    get_json(&format!("{}/lodgings/{}/periods", BASE, lodging_id)).await
}

pub async fn upsert_period(lodging_id: &str, req: &LodgingPeriodRequest) -> Result<LodgingPeriodResponse, String> {
    put_json(&format!("{}/lodgings/{}/periods", BASE, lodging_id), req).await
}

pub async fn request_rent_change(lodging_id: &str, req: &RentChangeRequest) -> Result<RentChangeResponse, String> {
    put_json(&format!("{}/lodgings/{}/rent-change", BASE, lodging_id), req).await
}

pub async fn approve_rent_change(lodging_id: &str, change_id: &str) -> Result<RentChangeResponse, String> {
    post_empty(&format!("{}/lodgings/{}/rent-change/{}/approve", BASE, lodging_id, change_id)).await
}

pub async fn reject_rent_change(lodging_id: &str, change_id: &str) -> Result<RentChangeResponse, String> {
    post_empty(&format!("{}/lodgings/{}/rent-change/{}/reject", BASE, lodging_id, change_id)).await
}

// ── Inventory ──
pub async fn list_lots(facility_id: Option<&str>, near_expiry: bool) -> Result<Vec<LotResponse>, String> {
    let mut url = format!("{}/inventory/lots?near_expiry={}", BASE, near_expiry);
    if let Some(fid) = facility_id {
        url.push_str(&format!("&facility_id={}", fid));
    }
    get_json(&url).await
}

pub async fn get_lot(id: &str) -> Result<LotResponse, String> {
    get_json(&format!("{}/inventory/lots/{}", BASE, id)).await
}

pub async fn create_lot(req: &CreateLotRequest) -> Result<LotResponse, String> {
    post_json(&format!("{}/inventory/lots", BASE), req).await
}

pub async fn reserve_lot(id: &str, req: &ReserveRequest) -> Result<LotResponse, String> {
    post_json(&format!("{}/inventory/lots/{}/reserve", BASE, id), req).await
}

pub async fn list_transactions(
    lot_id: Option<&str>, direction: Option<&str>, from_date: Option<&str>, to_date: Option<&str>
) -> Result<Vec<TransactionResponse>, String> {
    let mut url = format!("{}/inventory/transactions?", BASE);
    if let Some(lid) = lot_id { url.push_str(&format!("lot_id={}&", lid)); }
    if let Some(d) = direction { url.push_str(&format!("direction={}&", d)); }
    if let Some(f) = from_date { url.push_str(&format!("from_date={}&", f)); }
    if let Some(t) = to_date { url.push_str(&format!("to_date={}&", t)); }
    get_json(&url).await
}

pub async fn create_transaction(req: &CreateTransactionRequest) -> Result<TransactionResponse, String> {
    post_json(&format!("{}/inventory/transactions", BASE), req).await
}

pub fn audit_print_url(lot_id: &str) -> String {
    format!("{}/inventory/transactions/audit-print?lot_id={}", BASE, lot_id)
}

// ── Import/Export ──
pub async fn get_import_job(id: &str) -> Result<ImportJobResponse, String> {
    get_json(&format!("{}/import/jobs/{}", BASE, id)).await
}

pub async fn request_export(req: &ExportRequestBody) -> Result<ExportApprovalResponse, String> {
    post_json(&format!("{}/export/request", BASE), req).await
}

pub async fn approve_export(id: &str) -> Result<ExportApprovalResponse, String> {
    post_empty(&format!("{}/export/approve/{}", BASE, id)).await
}

pub fn export_download_url(id: &str) -> String {
    format!("{}/export/download/{}", BASE, id)
}

// ── Health ──
pub async fn health() -> Result<HealthResponse, String> {
    get_json(&format!("{}/health", BASE)).await
}

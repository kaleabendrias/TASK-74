//! Data transfer objects mirroring all backend API request/response types.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// Auth
#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub csrf_token: String,
    pub mfa_required: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    Administrator,
    Publisher,
    Reviewer,
    Clinician,
    InventoryClerk,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Administrator => write!(f, "Administrator"),
            Self::Publisher => write!(f, "Publisher"),
            Self::Reviewer => write!(f, "Reviewer"),
            Self::Clinician => write!(f, "Clinician"),
            Self::InventoryClerk => write!(f, "Inventory Clerk"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub role: UserRole,
    pub facility_id: Option<String>,
    pub mfa_enabled: bool,
    pub created_at: String,
}

// API Error
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Vec<FieldError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

// Resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceResponse {
    pub id: String,
    pub title: String,
    pub category: Option<String>,
    pub tags: serde_json::Value,
    pub hours: serde_json::Value,
    pub pricing: serde_json::Value,
    pub media_refs: serde_json::Value,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub state: String,
    pub scheduled_publish_at: Option<String>,
    pub current_version: i32,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateResourceRequest {
    pub title: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub hours: serde_json::Value,
    pub pricing: serde_json::Value,
    pub address: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub media_refs: Vec<String>,
    pub scheduled_publish_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tz_offset_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateResourceRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_refs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_publish_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tz_offset_minutes: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

// Lodgings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodgingResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub amenities: serde_json::Value,
    pub facility_id: Option<String>,
    pub deposit_amount: Option<f64>,
    pub monthly_rent: Option<f64>,
    pub deposit_cap_validated: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateLodgingRequest {
    pub name: String,
    pub description: Option<String>,
    pub amenities: Vec<String>,
    pub facility_id: Option<String>,
    pub deposit_amount: Option<f64>,
    pub monthly_rent: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateLodgingRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amenities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facility_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deposit_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_rent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodgingPeriodResponse {
    pub id: String,
    pub lodging_id: String,
    pub start_date: String,
    pub end_date: String,
    pub min_nights: i32,
    pub max_nights: i32,
    pub vacancy: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LodgingPeriodRequest {
    pub start_date: String,
    pub end_date: String,
    pub min_nights: Option<i32>,
    pub max_nights: Option<i32>,
    pub vacancy: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RentChangeRequest {
    pub proposed_rent: f64,
    pub proposed_deposit: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentChangeResponse {
    pub id: String,
    pub lodging_id: String,
    pub proposed_rent: f64,
    pub proposed_deposit: f64,
    pub status: String,
    pub requested_by: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<String>,
    pub created_at: String,
}

// Inventory
#[derive(Debug, Clone, Deserialize)]
pub struct WarehouseResponse {
    pub id: String,
    pub facility_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BinResponse {
    pub id: String,
    pub warehouse_id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotResponse {
    pub id: String,
    pub facility_id: String,
    pub warehouse_id: String,
    pub bin_id: String,
    pub item_name: String,
    pub lot_number: String,
    pub quantity_on_hand: i32,
    pub quantity_reserved: i32,
    pub expiration_date: Option<String>,
    pub near_expiry: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateLotRequest {
    pub facility_id: String,
    pub warehouse_id: String,
    pub bin_id: String,
    pub item_name: String,
    pub lot_number: String,
    pub quantity_on_hand: i32,
    pub expiration_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReserveRequest {
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: String,
    pub lot_id: String,
    pub direction: String,
    pub quantity: i32,
    pub reason: Option<String>,
    pub performed_by: String,
    pub created_at: String,
    pub is_immutable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTransactionRequest {
    pub lot_id: String,
    pub direction: String,
    pub quantity: i32,
    pub reason: Option<String>,
}

// Resource Versions
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceVersionResponse {
    pub id: String,
    pub resource_id: String,
    pub version_number: i32,
    pub snapshot: serde_json::Value,
    pub changed_by: String,
    pub created_at: String,
}

// Media
#[derive(Debug, Clone, Deserialize)]
pub struct MediaFileResponse {
    pub id: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub uploaded_by: String,
    pub created_at: String,
}

// Import/Export
#[derive(Debug, Clone, Deserialize)]
pub struct ImportJobResponse {
    pub id: String,
    pub job_type: String,
    pub status: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub progress_percent: i16,
    pub retries: i32,
    pub failure_log: Option<String>,
    pub committed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportRequestBody {
    pub export_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportApprovalResponse {
    pub id: String,
    pub export_type: String,
    pub requested_by: String,
    pub approved_by: Option<String>,
    pub watermark_text: Option<String>,
    pub status: String,
    pub created_at: String,
}

// Health
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub service: String,
    pub version: String,
    pub uptime_secs: u64,
    pub database_connected: bool,
    pub disk_usage_bytes: Option<u64>,
    pub config_profile: String,
}

// Toast
#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u32,
    pub kind: ToastKind,
    pub message: String,
}

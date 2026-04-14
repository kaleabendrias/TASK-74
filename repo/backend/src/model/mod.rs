use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ────────────────────────────────────────
// Enums
// ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Administrator,
    Publisher,
    Reviewer,
    Clinician,
    InventoryClerk,
}

impl UserRole {
    /// Returns the string representation of this role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Administrator => "Administrator",
            Self::Publisher => "Publisher",
            Self::Reviewer => "Reviewer",
            Self::Clinician => "Clinician",
            Self::InventoryClerk => "InventoryClerk",
        }
    }

    /// Parses a string into a `UserRole`, returning `None` if unrecognized.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Administrator" => Some(Self::Administrator),
            "Publisher" => Some(Self::Publisher),
            "Reviewer" => Some(Self::Reviewer),
            "Clinician" => Some(Self::Clinician),
            "InventoryClerk" => Some(Self::InventoryClerk),
            _ => None,
        }
    }
}

// ────────────────────────────────────────
// Auth DTOs
// ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub csrf_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_required: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub facility_id: Option<Uuid>,
    pub mfa_enabled: bool,
    pub created_at: DateTime<Utc>,
}

// ────────────────────────────────────────
// Health
// ────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub service: String,
    pub version: String,
    pub uptime_secs: u64,
    pub database_connected: bool,
    pub disk_usage_bytes: Option<u64>,
    pub config_profile: String,
}

// ────────────────────────────────────────
// Resources
// ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateResourceRequest {
    pub title: String,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub hours: serde_json::Value,
    #[serde(default)]
    pub pricing: serde_json::Value,
    pub address: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(default)]
    pub media_refs: Vec<Uuid>,
    pub scheduled_publish_at: Option<String>,
    pub contact_info: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateResourceRequest {
    pub title: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub hours: Option<serde_json::Value>,
    pub pricing: Option<serde_json::Value>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub media_refs: Option<Vec<Uuid>>,
    pub state: Option<String>,
    pub scheduled_publish_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceResponse {
    pub id: Uuid,
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
    pub scheduled_publish_at: Option<DateTime<Utc>>,
    pub current_version: i32,
    pub created_by: Uuid,
    pub facility_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub state: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub facility: Option<Uuid>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

// ────────────────────────────────────────
// Lodgings
// ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateLodgingRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub amenities: Vec<String>,
    pub facility_id: Option<Uuid>,
    pub deposit_amount: Option<f64>,
    pub monthly_rent: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLodgingRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub amenities: Option<Vec<String>>,
    pub facility_id: Option<Uuid>,
    pub deposit_amount: Option<f64>,
    pub monthly_rent: Option<f64>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LodgingResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub state: String,
    pub amenities: serde_json::Value,
    pub facility_id: Option<Uuid>,
    pub deposit_amount: Option<f64>,
    pub monthly_rent: Option<f64>,
    pub deposit_cap_validated: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LodgingPeriodRequest {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_nights: Option<i32>,
    pub max_nights: Option<i32>,
    pub vacancy: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct LodgingPeriodResponse {
    pub id: Uuid,
    pub lodging_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub min_nights: i32,
    pub max_nights: i32,
    pub vacancy: bool,
}

#[derive(Debug, Deserialize)]
pub struct RentChangeRequest {
    pub proposed_rent: f64,
    pub proposed_deposit: f64,
}

#[derive(Debug, Serialize)]
pub struct RentChangeResponse {
    pub id: Uuid,
    pub lodging_id: Uuid,
    pub proposed_rent: f64,
    pub proposed_deposit: f64,
    pub status: String,
    pub requested_by: Uuid,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ────────────────────────────────────────
// Inventory
// ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateLotRequest {
    pub facility_id: Uuid,
    pub warehouse_id: Uuid,
    pub bin_id: Uuid,
    pub item_name: String,
    pub lot_number: String,
    pub quantity_on_hand: i32,
    pub expiration_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct LotResponse {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub warehouse_id: Uuid,
    pub bin_id: Uuid,
    pub item_name: String,
    pub lot_number: String,
    pub quantity_on_hand: i32,
    pub quantity_reserved: i32,
    pub expiration_date: Option<NaiveDate>,
    pub near_expiry: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LotQuery {
    pub facility_id: Option<Uuid>,
    pub near_expiry: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ReserveRequest {
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub lot_id: Uuid,
    pub direction: String,
    pub quantity: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub direction: String,
    pub quantity: i32,
    pub reason: Option<String>,
    pub performed_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub is_immutable: bool,
}

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub lot_id: Option<Uuid>,
    pub direction: Option<String>,
    pub performed_by: Option<Uuid>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

// ────────────────────────────────────────
// Media
// ────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MediaFileResponse {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

// ────────────────────────────────────────
// Import / Export
// ────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ImportJobResponse {
    pub id: Uuid,
    pub job_type: String,
    pub status: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub progress_percent: i16,
    pub retries: i32,
    pub failure_log: Option<String>,
    pub committed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub export_type: String,
}

#[derive(Debug, Serialize)]
pub struct ExportApprovalResponse {
    pub id: Uuid,
    pub export_type: String,
    pub requested_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub watermark_text: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ────────────────────────────────────────
// Connector
// ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ConnectorPayload {
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ConnectorAck {
    pub accepted: bool,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
}

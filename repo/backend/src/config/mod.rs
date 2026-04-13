use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub tls: TlsConfig,
    pub auth: AuthConfig,
    pub crypto: CryptoConfig,
    pub totp: TotpConfig,
    pub uploads: UploadConfig,
    pub features: FeatureFlags,
    pub maintenance: MaintenanceConfig,
    pub prometheus: PrometheusConfig,
    pub canary: CanaryConfig,
    pub app: AppMetaConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub bind_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_conn")]
    pub max_connections: u32,
    #[serde(default = "default_min_conn")]
    pub min_connections: u32,
    #[serde(default = "default_timeout")]
    pub connect_timeout_secs: u64,
}

fn default_max_conn() -> u32 { 10 }
fn default_min_conn() -> u32 { 2 }
fn default_timeout() -> u64 { 5 }

#[derive(Debug, Clone, Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub hmac_secret: String,
    pub request_signing_key: String,
    #[serde(default = "default_session_ttl")]
    pub session_ttl_secs: u64,
    #[serde(default = "default_csrf_ttl")]
    pub csrf_token_ttl_secs: u64,
    pub argon2: Argon2Config,
}

fn default_session_ttl() -> u64 { 28800 } // 8 hours
fn default_csrf_ttl() -> u64 { 3600 }

#[derive(Debug, Clone, Deserialize)]
pub struct Argon2Config {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub output_len: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CryptoConfig {
    pub aes256_master_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TotpConfig {
    pub issuer: String,
    #[serde(default = "default_digits")]
    pub digits: u32,
    #[serde(default = "default_period")]
    pub period_secs: u64,
}

fn default_digits() -> u32 { 6 }
fn default_period() -> u64 { 30 }

#[derive(Debug, Clone, Deserialize)]
pub struct UploadConfig {
    pub max_size_bytes: usize,
    pub allowed_mimes: Vec<String>,
    #[serde(default = "default_storage_path")]
    pub storage_path: String,
}

fn default_storage_path() -> String { "/app/uploads".to_string() }

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureFlags {
    pub mfa_enabled: bool,
    pub csv_import: bool,
    pub export_watermark: bool,
    pub lodging_deposit_cap: bool,
    pub canary_release: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaintenanceConfig {
    pub window_cron: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrometheusConfig {
    pub scrape_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CanaryConfig {
    pub profile: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppMetaConfig {
    pub config_profile: String,
    pub service_name: String,
    pub version: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let builder = config::Config::builder()
            .add_source(config::File::with_name("config.toml").required(false))
            .add_source(config::Environment::with_prefix("").separator("_"))
            .build()
            .expect("Failed to build configuration");

        builder
            .try_deserialize()
            .expect("Failed to deserialize configuration")
    }
}

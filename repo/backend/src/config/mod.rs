use serde::Deserialize;

use ::config as config_crate;

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
    #[serde(default)]
    pub mq: MqConfig,
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

/// Optional on-prem Message Queue connector configuration.
///
/// Two transports are supported; they share the same HMAC-signed message envelope
/// so the signing key (`auth.request_signing_key`) covers both.
///
/// ## TCP transport (default)
///
/// Set `mq.enabled = true` and leave `mq.amqp_url` unset.
/// The application binds a raw TCP socket on `mq.bind_address`.
/// On-prem systems connect and send UTF-8 newline-terminated JSON objects,
/// one message per line:
/// ```json
/// {"Authorization":"<hmac>","X-Nonce":"<uuid>","X-Timestamp":"<unix_ts>","body":{...}}\n
/// ```
///
/// ## AMQP transport (RabbitMQ / compatible broker)
///
/// Set `mq.enabled = true` and provide `mq.amqp_url`
/// (e.g. `amqp://user:pass@rabbitmq:5672/%2F`).
/// Set `mq.amqp_queue` to the queue name the consumer should read from
/// (defaults to `"tourism_inbound"`).
/// Each AMQP message payload must be the same JSON envelope as above
/// (without the trailing newline).  The consumer acks on success and
/// nacks (with `requeue = false`) on validation failure.
#[derive(Debug, Clone, Deserialize)]
pub struct MqConfig {
    /// Whether to start the MQ connector. Defaults to false.
    #[serde(default)]
    pub enabled: bool,
    /// TCP bind address used when `amqp_url` is not set. Defaults to `127.0.0.1:9999`.
    #[serde(default = "default_mq_bind")]
    pub bind_address: String,
    /// AMQP broker URL (e.g. `amqp://user:pass@host:5672/%2F`).
    /// When set, the AMQP consumer is used instead of the raw TCP listener.
    pub amqp_url: Option<String>,
    /// Name of the AMQP queue to consume from. Defaults to `"tourism_inbound"`.
    #[serde(default = "default_amqp_queue")]
    pub amqp_queue: String,
}

fn default_mq_bind() -> String { "127.0.0.1:9999".to_string() }
fn default_amqp_queue() -> String { "tourism_inbound".to_string() }

impl Default for MqConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_address: default_mq_bind(),
            amqp_url: None,
            amqp_queue: default_amqp_queue(),
        }
    }
}

impl AppConfig {
    /// Loads application configuration from `config.toml` and environment variables.
    pub fn load() -> Self {
        let builder = config_crate::Config::builder()
            .add_source(config_crate::File::with_name("config.toml").required(false))
            .add_source(config_crate::Environment::with_prefix("").separator("_"))
            .build()
            .expect("Failed to build configuration");

        builder
            .try_deserialize()
            .expect("Failed to deserialize configuration")
    }
}

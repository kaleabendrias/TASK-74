//! Optional on-prem Message Queue (MQ) connector.
//!
//! This module provides a trait-based interface so the connector logic can be
//! shared across transports, and two concrete transport implementations:
//!
//! * [`spawn_mq_listener`] — raw TCP socket, newline-delimited JSON (no broker
//!   required; suitable for isolated on-prem networks).
//! * [`spawn_amqp_consumer`] — AMQP 0-9-1 consumer backed by a RabbitMQ-compatible
//!   broker; preferred when a message broker is already available.
//!
//! # Wire format (both transports)
//!
//! Each message payload is a UTF-8 JSON object:
//!
//! ```json
//! {"Authorization":"<hmac>","X-Nonce":"<uuid>","X-Timestamp":"<unix_epoch>","body":{...}}
//! ```
//!
//! The `Authorization` value, `X-Nonce`, and `X-Timestamp` follow the same
//! validation rules as the REST `/api/connector/inbound` endpoint so that a
//! single signing key covers all transports.
//!
//! # Security
//!
//! * Bind the TCP listener to `127.0.0.1` (loopback) or an internal VLAN address —
//!   never expose it on a public interface without a firewall rule.
//! * Use TLS-enabled AMQP (`amqps://`) in production to protect messages in transit.
//! * The HMAC secret (`auth.request_signing_key`) must be rotated like any
//!   other credential.  Do **not** use the default development value in
//!   production.
//! * Replay protection is enforced via the nonce table — identical to the REST
//!   connector.

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use futures_util::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Arc;
use tracing;

use crate::service::connector as svc;

type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Envelope that on-prem clients send over the TCP MQ transport.
/// Field names mirror the HTTP header names used by the REST connector.
#[derive(Debug, Deserialize)]
struct MqEnvelope {
    #[serde(rename = "Authorization")]
    pub authorization: String,
    #[serde(rename = "X-Nonce")]
    pub nonce: String,
    #[serde(rename = "X-Timestamp")]
    pub timestamp: String,
    pub body: serde_json::Value,
}

/// Trait that abstracts a message-queue transport so the processing logic can
/// be tested or swapped without touching business code.
pub trait MqConnector: Send + Sync {
    /// Process a single raw message.  Returns `Ok(ack_json)` on success or an
    /// error string that will be sent back to the caller.
    fn handle(&self, raw: &[u8]) -> Result<serde_json::Value, String>;
}

/// The canonical MQ connector implementation, backed by the same HMAC-signed
/// validation pipeline used by the REST `inbound` endpoint.
pub struct HmacMqConnector {
    pool: DbPool,
    signing_key: String,
}

impl HmacMqConnector {
    pub fn new(pool: DbPool, signing_key: String) -> Self {
        Self { pool, signing_key }
    }
}

impl MqConnector for HmacMqConnector {
    fn handle(&self, raw: &[u8]) -> Result<serde_json::Value, String> {
        let envelope: MqEnvelope = serde_json::from_slice(raw)
            .map_err(|e| format!("envelope parse error: {}", e))?;

        let body_bytes = serde_json::to_vec(&envelope.body)
            .map_err(|e| format!("body serialization error: {}", e))?;

        let mut conn = self.pool.get()
            .map_err(|e| format!("db pool error: {}", e))?;

        let ack = svc::validate_and_process(
            &mut conn,
            &self.signing_key,
            &envelope.authorization,
            &body_bytes,
            &envelope.nonce,
            &envelope.timestamp,
            "mq://local",
        )
        .map_err(|e| format!("{}", e.body.message))?;

        serde_json::to_value(ack).map_err(|e| e.to_string())
    }
}

/// Spawns an AMQP consumer as a background Tokio task.
///
/// Connects to the broker at `amqp_url`, declares a durable queue named
/// `queue_name` (idempotent — safe to call against an existing queue), and
/// consumes messages one at a time.  Each delivery payload is passed to
/// `connector.handle`; on success the message is acknowledged, on failure it
/// is nacked with `requeue = false` so the broker can route it to a dead-letter
/// exchange if one is configured.
///
/// The task attempts to reconnect indefinitely on connection loss, backing off
/// by 5 seconds between attempts so a transient broker restart does not spin
/// the CPU.
pub fn spawn_amqp_consumer(
    amqp_url: String,
    queue_name: String,
    connector: Arc<dyn MqConnector>,
) {
    tokio::spawn(async move {
        loop {
            match run_amqp_consumer(&amqp_url, &queue_name, Arc::clone(&connector)).await {
                Ok(()) => {
                    tracing::info!(queue = %queue_name, "AMQP consumer exited cleanly — reconnecting");
                }
                Err(e) => {
                    tracing::error!(queue = %queue_name, error = %e, "AMQP consumer error — reconnecting in 5 s");
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });
}

async fn run_amqp_consumer(
    amqp_url: &str,
    queue_name: &str,
    connector: Arc<dyn MqConnector>,
) -> Result<(), lapin::Error> {
    let conn = Connection::connect(amqp_url, ConnectionProperties::default()).await?;
    tracing::info!(queue = %queue_name, "AMQP connection established");

    let channel = conn.create_channel().await?;

    // Declare the queue as durable so messages survive broker restarts.
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    // One message at a time: do not pre-fetch more until the current one is acked.
    channel
        .basic_qos(1, BasicQosOptions { global: false })
        .await?;

    let mut consumer = channel
        .basic_consume(
            queue_name,
            "tourism-backend",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tracing::info!(queue = %queue_name, "AMQP consumer started");

    while let Some(delivery_result) = consumer.next().await {
        match delivery_result {
            Ok(delivery) => {
                let tag = delivery.delivery_tag;
                match connector.handle(&delivery.data) {
                    Ok(ack) => {
                        tracing::debug!(queue = %queue_name, ack = %ack, "AMQP message accepted");
                        let _ = channel
                            .basic_ack(tag, BasicAckOptions::default())
                            .await;
                    }
                    Err(msg) => {
                        tracing::warn!(queue = %queue_name, error = %msg, "AMQP message rejected — nacking");
                        let _ = channel
                            .basic_nack(tag, BasicNackOptions { requeue: false, ..Default::default() })
                            .await;
                    }
                }
            }
            Err(e) => {
                tracing::error!(queue = %queue_name, error = %e, "AMQP delivery error");
                return Err(e);
            }
        }
    }

    Ok(())
}

/// Spawns the local TCP MQ listener as a background Tokio task.
///
/// The listener accepts one connection at a time, reads newline-delimited JSON
/// messages, processes each through `connector`, and writes a single-line JSON
/// acknowledgement back before closing the connection.
pub fn spawn_mq_listener(bind_address: String, connector: Arc<dyn MqConnector>) {
    std::thread::spawn(move || {
        let listener = match TcpListener::bind(&bind_address) {
            Ok(l) => {
                tracing::info!(address = %bind_address, "MQ listener started");
                l
            }
            Err(e) => {
                tracing::error!(error = %e, address = %bind_address, "Failed to bind MQ listener");
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
                    let connector = Arc::clone(&connector);
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stream.try_clone().expect("stream clone"));
                        for line in reader.lines() {
                            let line = match line {
                                Ok(l) if !l.trim().is_empty() => l,
                                Ok(_) => continue,
                                Err(e) => {
                                    tracing::warn!(peer = %peer, error = %e, "MQ read error");
                                    break;
                                }
                            };

                            let response = match connector.handle(line.as_bytes()) {
                                Ok(ack) => {
                                    tracing::debug!(peer = %peer, "MQ message accepted");
                                    serde_json::json!({"ok": true, "ack": ack})
                                }
                                Err(msg) => {
                                    tracing::warn!(peer = %peer, error = %msg, "MQ message rejected");
                                    serde_json::json!({"ok": false, "error": msg})
                                }
                            };

                            let mut out = serde_json::to_string(&response).unwrap_or_default();
                            out.push('\n');
                            if stream.write_all(out.as_bytes()).is_err() {
                                break;
                            }
                        }
                    });
                }
                Err(e) => tracing::warn!(error = %e, "MQ accept error"),
            }
        }
    });
}

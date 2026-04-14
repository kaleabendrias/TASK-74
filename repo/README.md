# Regional Tourism Resource & Lodging Operations Portal

A full-stack web application for managing tourism resources, lodging accommodations, inventory, and operational workflows across multiple facilities. The portal provides role-based access for Administrators, Publishers, Reviewers, Clinicians, and Inventory Clerks, supporting the full content lifecycle from draft creation through reviewer approval to publication, lodging management with deposit validation and rent-change workflows, facility-scoped inventory tracking with near-expiry alerts, bulk Excel import with progress tracking, watermarked export approvals, HMAC-signed connector integrations, and Prometheus-compatible observability — all deployed as three Docker services with a single command.

---

## Prerequisites

| Requirement | Version |
|---|---|
| **Docker** | 20.10+ |
| **Docker Compose** | 2.0+ (V2 plugin) |

No other tools, runtimes, or package managers are required. Everything builds and runs inside containers.

---

## Quick Start

```bash
docker compose up --build
```

Once all three services are healthy (typically 2-3 minutes for the first build):

| Endpoint | URL |
|---|---|
| Frontend (Yew SPA) | [http://localhost:8081](http://localhost:8081) |
| Backend API | [http://localhost:8080](http://localhost:8080) |
| Health Check | [http://localhost:8080/api/health](http://localhost:8080/api/health) |
| Prometheus Metrics | [http://localhost:8080/api/metrics](http://localhost:8080/api/metrics) |

Log in at the frontend with any of the default users listed below. The backend automatically seeds default users and a facility on first startup when the database is empty.

---

## Services

| Service | Image | Internal Port | Host Port | Purpose |
|---|---|---|---|---|
| `db` | `postgres:16-alpine` | 5432 | 5433 | PostgreSQL database with all schema migrations applied at startup |
| `backend` | Multi-stage Rust build (scratch) | 8080 | 8080 | Actix-web REST API with TLS certs, embedded migrations, background job runner |
| `frontend` | Trunk-served Yew WASM | 8081 | 8081 | Single-page application proxying `/api/*` requests to the backend |

---

## Default Users

The backend seeds these users automatically on first startup:

| Username | Role | Facility | Notes |
|---|---|---|---|
| `admin` | Administrator | All | Password set via `INIT_ADMIN_PASSWORD` env var (default: `changeme`) |
| `publisher` | Publisher | All | Same initial password |
| `reviewer` | Reviewer | All | Same initial password |
| `clinician` | Clinician | Main Facility | Same initial password |
| `clerk` | InventoryClerk | Main Facility | Same initial password |

All accounts share the initial password defined by `INIT_ADMIN_PASSWORD`. Change passwords after first login.

---

## Testing

Run the full test suite (unit + API integration tests with coverage reporting):

```bash
./run_tests.sh
```

The script:
1. Builds and starts test containers using the `test` profile in `docker-compose.yml`
2. Waits for the PostgreSQL test database to become healthy
3. Runs **145 unit tests** covering all business logic and cryptographic functions
4. Runs **50 API integration tests** against a real database and running backend
5. Reports coverage for both suites independently

Expected output:

```
┌──────────────────┬────────────┬──────────┬────────┐
│ Suite            │ Coverage   │ Required │ Status │
├──────────────────┼────────────┼──────────┼────────┤
│ Unit Tests       │    9X.X%%  │    90%%  │ PASS   │
│ API Tests        │    9X.X%%  │    90%%  │ PASS   │
└──────────────────┴────────────┴──────────┴────────┘
```

Both suites must independently achieve >= 90% coverage. The script tears down all containers regardless of pass/fail.

### Coverage (Optional)

To generate a coverage report with [cargo-tarpaulin](https://github.com/xd009642/tarpaulin):

```bash
docker compose --profile test run --rm test-runner bash -c \
  'cargo tarpaulin -p unit_tests -p api_tests --out Html --output-dir /coverage -- --test-threads=1'
```

The HTML report will be in `coverage_output/tarpaulin-report.html`.

---

## Architecture

### Backend (Actix-web / Diesel / PostgreSQL)

The backend follows a strict layered architecture where each layer depends only on layers below it:

```
api/          Actix route handlers and extractors (HTTP concerns only)
  |
service/      Pure business logic, validation, state machines
  |
repository/   Diesel-based PostgreSQL query modules
  |
model/        Domain structs, enums, DTOs
schema/       Diesel table definitions (21 tables)
```

Additional modules:
- `crypto/` — Argon2id, AES-256-GCM, TOTP, HMAC-SHA256, CSRF, SHA-256
- `middleware/` — Session-cookie extractor (`RbacContext`), `require_role!` macro
- `jobs/` — Background import job runner (tokio::spawn polling loop)
- `config/` — Strongly-typed TOML + env-var deserialization

### Frontend (Yew / WASM)

```
pages/        Route-level page components (login, dashboard, resources, etc.)
  |
components/   Reusable UI components (sidebar, toast, route guard, app shell)
  |
services/     API client (fetch-based with CSRF injection), masking utilities
  |
models/       TypeScript-style DTOs mirroring backend responses
context/      AuthProvider + ToastProvider (Yew context/reducer)
```

### System Diagram

```mermaid
graph LR
    subgraph Docker Network
        FE[Frontend<br/>Yew WASM<br/>:8081]
        BE[Backend<br/>Actix-web<br/>:8080]
        DB[(PostgreSQL 16<br/>:5432)]
    end

    User((Browser)) -->|HTTP :8081| FE
    FE -->|/api/* proxy| BE
    BE -->|Diesel| DB
    BE -->|Background Jobs| BE
```

---

## Security

### Authentication
- **Argon2id** password hashing with configurable memory/iterations/parallelism
- **Optional TOTP MFA** (30-second step, SHA-1, via totp-rs) with encrypted secret storage
- **HttpOnly / Secure / SameSite=Strict** session cookies (8-hour default expiry)
- **CSRF tokens** returned in response body, required as `X-CSRF-Token` header

### Authorization
- **Role-based access control** via `RbacContext` Actix extractor and `require_role!` macro
- **Menu visibility** — sidebar items rendered conditionally per role
- **Route guards** — frontend redirects unauthorized roles to /forbidden
- **Data-scope filtering** — Clinicians and InventoryClerks see only their facility's data

### Data Protection
- **AES-256-GCM column encryption** for sensitive fields (TOTP secrets, contact info)
- **Self-signed TLS certificates** generated at Docker build time
- **Upload validation** — extension allowlist + MIME sniffing via `infer` crate to reject mismatched types
- **Data masking** — phone numbers and emails rendered with partial redaction on the frontend
- **SHA-256 file checksums** computed and stored on every upload

### Compliance Features
- **Immutable transaction logs** — inventory transactions are insert-only with `is_immutable` flag
- **Second-person export approval** — exports require a different user to approve before download
- **Watermarked exports** — approved exports embed the approver's username and timestamp
- **Anti-replay connectors** — inbound payloads verified via HMAC-SHA256, with 5-minute timestamp window and nonce deduplication via `idempotency_keys`
- **Audit log** — every significant action recorded with actor, entity, detail, and IP

---

## API Overview

All endpoints are under `/api`. Authentication is required unless noted.

| Group | Method | Path | Roles | Description |
|---|---|---|---|---|
| **Health** | GET | `/api/health` | Public | Service health, DB connectivity, uptime |
| **Metrics** | GET | `/api/metrics` | Authenticated | Prometheus-format metrics |
| **Auth** | POST | `/api/auth/login` | Public | Login with username/password (+optional TOTP) |
| | POST | `/api/auth/logout` | Authenticated | Destroy session |
| | GET | `/api/auth/me` | Authenticated | Current user profile |
| **Resources** | POST | `/api/resources` | Admin, Publisher | Create resource (draft) |
| | GET | `/api/resources` | Admin, Publisher, Reviewer | Paginated list with filters |
| | GET | `/api/resources/:id` | Admin, Publisher, Reviewer | Resource detail |
| | PUT | `/api/resources/:id` | Admin, Publisher, Reviewer | Update resource / transition state |
| **Lodgings** | POST | `/api/lodgings` | Admin, Publisher | Create lodging |
| | GET | `/api/lodgings` | Admin, Publisher, Reviewer | List lodgings |
| | GET | `/api/lodgings/:id` | Admin, Publisher, Reviewer | Lodging detail |
| | PUT | `/api/lodgings/:id` | Admin, Publisher, Reviewer | Update lodging |
| | GET | `/api/lodgings/:id/periods` | Authenticated | List vacancy periods |
| | PUT | `/api/lodgings/:id/periods` | Admin, Publisher | Add vacancy period |
| | PUT | `/api/lodgings/:id/rent-change` | Admin, Publisher | Request rent change |
| | POST | `/api/lodgings/:id/rent-change/:cid/approve` | Admin, Reviewer | Approve rent change |
| | POST | `/api/lodgings/:id/rent-change/:cid/reject` | Admin, Reviewer | Reject rent change |
| **Inventory** | POST | `/api/inventory/lots` | Admin, Clerk | Create lot |
| | GET | `/api/inventory/lots` | Admin, Clerk, Clinician | List lots (with `?near_expiry=true`) |
| | GET | `/api/inventory/lots/:id` | Admin, Clerk, Clinician | Lot detail |
| | POST | `/api/inventory/lots/:id/reserve` | Admin, Clerk | Reserve stock |
| | POST | `/api/inventory/transactions` | Admin, Clerk | Record transaction |
| | GET | `/api/inventory/transactions` | Admin, Clerk, Clinician | List transactions (filterable) |
| | GET | `/api/inventory/transactions/audit-print` | Admin, Clerk | Printable HTML audit trail |
| **Media** | POST | `/api/media/upload` | Admin, Publisher | Multipart file upload |
| | GET | `/api/media/:id/download` | Authenticated | Download file |
| **Import** | POST | `/api/import/upload` | Admin, Clerk | Upload .xlsx for import |
| | GET | `/api/import/jobs/:id` | Authenticated | Job status and progress |
| **Export** | POST | `/api/export/request` | Authenticated | Request export |
| | POST | `/api/export/approve/:id` | Admin, Reviewer | Approve export |
| | GET | `/api/export/download/:id` | Authenticated | Download approved export |
| **Connector** | POST | `/api/connector/inbound` | HMAC-signed | Ingest external payloads |

---

## On-Prem MQ Connector

The backend ships an optional message queue connector that lets on-prem systems
push payloads into the same processing pipeline as the REST
`POST /api/connector/inbound` endpoint.  Two transports are supported; both
share the same HMAC-signed envelope so a single signing key covers all surfaces.

### Wire Format

Every message — regardless of transport — must be a JSON object:

```json
{
  "Authorization": "sha256=<hmac-hex>",
  "X-Nonce": "<uuid-v4>",
  "X-Timestamp": "<unix-epoch-seconds>",
  "body": { ... }
}
```

| Field | Description |
|---|---|
| `Authorization` | `sha256=` prefix followed by the lowercase hex HMAC-SHA256 of the raw `body` bytes, signed with `auth.request_signing_key` |
| `X-Nonce` | UUID v4; must be unique within the replay-protection window (stored in `idempotency_keys`) |
| `X-Timestamp` | Unix epoch seconds; rejected if older than 5 minutes or in the future |
| `body` | Arbitrary JSON payload passed to the inbound connector service |

### TCP Transport (no broker required)

Suitable for isolated networks where installing a broker is not feasible.

**Config (`config.toml` or environment variables):**

```toml
[mq]
enabled       = true
bind_address  = "127.0.0.1:9999"   # loopback or internal VLAN — never a public interface
```

| Env var | Equivalent TOML key |
|---|---|
| `MQ_ENABLED=true` | `mq.enabled` |
| `MQ_BIND_ADDRESS=127.0.0.1:9999` | `mq.bind_address` |

**Connection:** Open a TCP connection to `<bind_address>` and send one JSON object
per line (terminated by `\n`).  The server replies with a single JSON line:

```json
{"ok": true,  "ack": { ... }}   // success
{"ok": false, "error": "..."}   // validation or processing failure
```

### AMQP Transport (RabbitMQ / compatible broker)

Preferred when a message broker is already available. Requires a
RabbitMQ-compatible AMQP 0-9-1 broker.

**Config:**

```toml
[mq]
enabled    = true
amqp_url   = "amqp://user:pass@rabbitmq:5672/%2F"   # use amqps:// in production
amqp_queue = "tourism_inbound"                       # defaults to "tourism_inbound"
```

| Env var | Equivalent TOML key |
|---|---|
| `MQ_ENABLED=true` | `mq.enabled` |
| `MQ_AMQP_URL=amqp://...` | `mq.amqp_url` |
| `MQ_AMQP_QUEUE=tourism_inbound` | `mq.amqp_queue` |

When `amqp_url` is set the AMQP consumer is started instead of the TCP listener.
The backend declares the queue as **durable** on startup (idempotent) and
consumes one message at a time (`basic_qos prefetch=1`).

* **Ack** — sent on successful validation and processing.
* **Nack (`requeue=false`)** — sent when HMAC validation fails or processing
  returns an error; configure a dead-letter exchange on the queue to capture
  rejected messages for inspection.

**Publishing from an on-prem system:**

```python
import pika, json, hmac, hashlib, time, uuid

signing_key = b"<your_request_signing_key>"
body = {"event": "stock_update", "item_id": "..."}
body_bytes = json.dumps(body, separators=(",", ":")).encode()
sig = hmac.new(signing_key, body_bytes, hashlib.sha256).hexdigest()

envelope = {
    "Authorization": f"sha256={sig}",
    "X-Nonce": str(uuid.uuid4()),
    "X-Timestamp": str(int(time.time())),
    "body": body,
}

conn = pika.BlockingConnection(pika.URLParameters("amqp://user:pass@rabbitmq:5672/%2F"))
ch = conn.channel()
ch.queue_declare(queue="tourism_inbound", durable=True)
ch.basic_publish(exchange="", routing_key="tourism_inbound",
                 body=json.dumps(envelope),
                 properties=pika.BasicProperties(delivery_mode=2))
conn.close()
```

### Security Notes

* **TCP transport:** bind only to `127.0.0.1` or a trusted internal VLAN address.
  Exposing the port on a public interface without a firewall rule is a security risk.
* **AMQP transport:** use `amqps://` (TLS) in production to encrypt messages in
  transit. Configure the broker to require authentication.
* **Signing key:** `auth.request_signing_key` must be at least 32 bytes of random
  entropy. Rotate it like any other credential. Do **not** use the default
  development value (`changeme`) in production.
* **Replay protection:** the nonce table (`idempotency_keys`) deduplicates
  messages. Identical nonces within the 5-minute timestamp window are rejected.

---

## Configuration

All configuration is embedded directly in `docker-compose.yml` environment blocks and `backend/config.toml`. No `.env` files are used.

### Configuration Sources (in precedence order)
1. **Environment variables** — set in `docker-compose.yml` for each service
2. **config.toml** — checked-in TOML file copied into the backend container at build time

### Key Configuration Groups

| Group | Variables | Purpose |
|---|---|---|
| Database | `DATABASE_URL` | PostgreSQL connection string |
| Auth | `HMAC_SECRET`, `REQUEST_SIGNING_KEY` | Session signing, connector verification |
| Argon2 | `ARGON2_MEMORY_KIB`, `ARGON2_ITERATIONS`, `ARGON2_PARALLELISM` | Password hashing cost |
| Encryption | `AES256_MASTER_KEY` | Column-level AES-256-GCM encryption |
| TOTP | `TOTP_ISSUER` | MFA issuer name shown in authenticator apps |
| Uploads | `UPLOAD_MAX_SIZE_BYTES`, `UPLOAD_ALLOWED_MIMES` | File upload constraints |
| Features | `FEATURE_MFA_ENABLED`, `FEATURE_CSV_IMPORT`, etc. | Feature switches |
| Maintenance | `MAINTENANCE_WINDOW_CRON` | Cron expression for maintenance windows |
| Prometheus | `PROMETHEUS_SCRAPE_PATH` | Metrics endpoint path |
| Canary | `CANARY_PROFILE` | Release profile selector (`stable`/`canary`) |
| Profile | `CONFIG_PROFILE` | Environment name (`development`/`test`/`production`) |

### Environment-Specific Profiles

The `CONFIG_PROFILE` variable identifies the active environment. The backend reads `config.toml` for defaults and allows any value to be overridden via environment variables. Profile-specific parameters can also be stored in the `config_parameters` database table and managed through the Configuration Center UI (Administrator only).

# Delivery Acceptance + Project Architecture Audit (Static-Only)

## 1) Scope, Method, and Constraints

- Audit mode: **static review only** (no app startup, no Docker, no test execution).
- Evidence source: code, configuration, migrations, frontend pages, and test source files.
- All conclusions are based on repository state under `repo/`.

---

## 2) Acceptance Section A — Architecture & Deployment Baseline

### Verdict
**PARTIAL PASS**

### Evidence (met)
- Layered backend structure is explicit (`api`, `service`, `repository`, `schema`, etc.): `backend/src/lib.rs:1-10`.
- Startup lifecycle includes migrations, seed defaults, job runner, and scheduled publisher: `backend/src/main.rs:72-76`.
- Structured JSON logging + request tracing middleware present: `backend/src/main.rs:61-67`, `backend/src/main.rs:117`.
- TLS is enforced in production profile (panic if certs missing), optional in non-production: `backend/src/main.rs:97-105`, `backend/src/main.rs:121-127`.
- On-prem connector mode is present (AMQP or TCP listener): `backend/src/main.rs:77-92`.

### Gaps / Risks
- Non-production can run plain HTTP without TLS; this is intentional but still a deployment risk if profiles are misconfigured: `backend/src/main.rs:97-105`, `backend/src/main.rs:124-127`.

---

## 3) Acceptance Section B — Resource & Content Lifecycle

### Verdict
**PASS**

### Evidence (met)
- Resource routes include create/list/get/update/version history: `backend/src/api/mod.rs:27-31`.
- State transition logic is tested via direct production function use (`draft -> in_review -> published -> offline` with role constraints): `unit_tests/src/test_state_machine.rs:9-105`.
- Resource review/publish transition coverage exists in API tests: `API_tests/src/test_resources.rs` (module present in `API_tests/src/lib.rs`).

### Gaps / Risks
- No major blocker found statically in this area.

---

## 4) Acceptance Section C — Lodgings, Availability, and Rent Workflow

### Verdict
**PARTIAL PASS**

### Evidence (met)
- Lodging CRUD/list/get with role checks and facility enforcement path exists: `backend/src/api/lodgings.rs:30-89`.
- Availability periods endpoint and overlap/min/max validation implemented: `backend/src/api/lodgings.rs:93-127`, `backend/src/service/lodgings.rs:148-197`.
- Rent change request + approve/reject flow implemented with review decision logging: `backend/src/api/lodgings.rs:131-200`, `backend/src/service/lodgings.rs:202-299`.
- Frontend has lodging form, period management, and rent-change UI actions: `frontend/src/pages/lodgings.rs:434-592`.

### Gaps / Risks
- Conflict resolution appears binary (approve/reject) with no explicit counterproposal data model/state in schema/service (only `pending/approved/rejected` behavior visible):
  - `backend/migrations/00000000000000_initial/up.sql:118-127`
  - `backend/src/service/lodgings.rs:217`, `backend/src/service/lodgings.rs:236-240`, `backend/src/service/lodgings.rs:281-285`

---

## 5) Acceptance Section D — Inventory, Auditability, and Facility Scoping

### Verdict
**PASS**

### Evidence (met)
- Facility scoping helper exists and is used across lot/transaction/audit operations: `backend/src/api/inventory.rs:63-72`, `backend/src/api/inventory.rs:81-83`, `backend/src/api/inventory.rs:147-150`, `backend/src/api/inventory.rs:200-203`.
- Cross-entity integrity checks (warehouse→facility, bin→warehouse) in lot creation: `backend/src/service/inventory.rs:29-47`.
- Reserve and transactions are transactional with immutable audit records: `backend/src/service/inventory.rs:94-111`, `backend/src/service/inventory.rs:133-165`.
- Audit print endpoint exists and emits transaction trail HTML: `backend/src/api/inventory.rs:191-208`, `backend/src/service/inventory.rs:200-249`.

### Gaps / Risks
- None high-severity found statically.

---

## 6) Acceptance Section E — Import/Export, Recovery, and Connector Ingress

### Verdict
**PARTIAL PASS**

### Evidence (met)
- Import job schema has progress, retries, status, failure log, staging metadata: `backend/migrations/00000000000000_initial/up.sql:209-225`.
- Job runner implements row cap, validation phase, resumable cursor, chunk progress updates: `backend/src/jobs/runner.rs:142-149`, `backend/src/jobs/runner.rs:197`, `backend/src/jobs/runner.rs:231-260`, `backend/src/jobs/runner.rs:290`.
- API/frontend expose import job progress and failures: `backend/src/api/mod.rs:78-80`, `frontend/src/pages/import_export.rs:253-283`.
- Export request/approve/download routes present: `backend/src/api/mod.rs:80-93`.
- Connector idempotency + conflict behavior covered in API tests: `API_tests/src/test_connector_envelope.rs:160-198`.

### Gaps / Risks
- Real-time status channel is polling-based from frontend (`Interval` every 2s), not WebSocket/event stream:
  - `frontend/src/pages/import_export.rs:4`
  - `frontend/src/pages/import_export.rs:44-66`
- No backend WebSocket handler surface found in static search of backend source.

---

## 7) Acceptance Section F — Security, RBAC, and Operational Controls

### Verdict
**PARTIAL PASS**

### Evidence (met)
- Session extraction + CSRF validation for mutating routes (with explicit exemptions) is centralized in extractor: `backend/src/middleware/auth_guard.rs:56-63`, `backend/src/middleware/auth_guard.rs:110-117`.
- RBAC checks enforced at handler boundary through `require_role!`: examples in `backend/src/api/config.rs:14`, `backend/src/api/inventory.rs:18`, `backend/src/api/lodgings.rs:36`.
- Password hashing, MFA flows, and security tests are present (auth/security suites in API tests, crypto unit tests modules in `unit_tests/src/lib.rs`).

### Gaps / Risks
- Health endpoint is unauthenticated and returns service metadata (version/profile/disk usage), which may exceed least exposure for production: `backend/src/api/mod.rs:18`, `backend/src/api/health.rs:18-25`.
- Metrics endpoint requires auth context but does not role-restrict; any authenticated user can read operational counters: `backend/src/api/metrics.rs:17-20`.
- Seed path writes generated initial admin password to `/tmp/.tourism_init_password`; this is practical for bootstrap but sensitive if host hardening is weak: `backend/src/lib.rs:63-70`.

---

## 8) Tests & Logging Review

### 8.1 Static Test Architecture Assessment
- Test crates are split into `unit_tests` and `API_tests` with broad module coverage declarations:
  - `unit_tests/src/lib.rs`
  - `API_tests/src/lib.rs`
- Orchestration script expects both suites and captures logs/artifacts: `run_tests.sh:81-115`, `run_tests.sh:147-179`.
- Logging posture in backend is structured JSON + tracing middleware: `backend/src/main.rs:61-67`, `backend/src/main.rs:117`.

### 8.2 Static Coverage Mapping Table

| Requirement / Risk Area | Implementation Evidence | Static Test Evidence | Coverage Confidence |
|---|---|---|---|
| Resource state machine + role transitions | `backend/src/service/resources.rs` + routes in `backend/src/api/mod.rs:27-31` | `unit_tests/src/test_state_machine.rs:9-105`, `API_tests/src/test_resources.rs` | High |
| Lodging period bounds + overlap rejection | `backend/src/service/lodgings.rs:156-185` | `unit_tests/src/test_night_bounds.rs`, `unit_tests/src/test_periods.rs`, `API_tests/src/test_lodgings.rs` | High |
| Deposit cap validation | `backend/src/service/lodgings.rs:40-43`, `backend/src/service/lodgings.rs:93-95`, `backend/src/service/lodgings.rs:210-212` | `unit_tests/src/test_deposit_cap.rs`, `API_tests/src/test_lodgings.rs` | High |
| Inventory reserve/transaction validation | `backend/src/service/inventory.rs:87-92`, `backend/src/service/inventory.rs:120-131` | `unit_tests/src/test_inventory_logic.rs:8-52`, `API_tests/src/test_inventory.rs` | High |
| Import resumable cursor/progress | `backend/src/jobs/runner.rs:126`, `backend/src/jobs/runner.rs:231-260` | `unit_tests/src/test_import_resumable.rs:53-108`, `API_tests/src/test_job_recovery.rs` | Medium-High |
| Import row validation + limits | `backend/src/jobs/runner.rs:143-149`, `backend/src/jobs/runner.rs:197` | `unit_tests/src/test_import_validation.rs:29-103`, `API_tests/src/test_import_export.rs` | Medium-High |
| Connector anti-replay/idempotency | `backend/src/repository/connector.rs:17-30` | `API_tests/src/test_connector.rs`, `API_tests/src/test_connector_envelope.rs:160-198` | High |
| CSRF/session protection | `backend/src/middleware/auth_guard.rs:73-85`, `backend/src/middleware/auth_guard.rs:110-117` | `API_tests/src/test_security.rs`, `API_tests/src/test_auth.rs` | High |
| Media file-type restrictions | `backend/src/service/media.rs` | `API_tests/src/test_media.rs:29-121` | High |
| Metrics/health operational endpoints | `backend/src/api/health.rs`, `backend/src/api/metrics.rs` | `API_tests/src/test_metrics_health.rs` | Medium |

---

## 9) Severity-Ranked Findings and Final Decision

## Findings (highest severity first)

1. **HIGH — Requirement gap: real-time import status channel appears polling-only, not WebSocket/event-based**  
   - Evidence: frontend uses periodic polling interval (`Interval::new(2_000)`) for import job updates (`frontend/src/pages/import_export.rs:44-66`).  
   - Impact: misses strict “real-time push channel” expectations where required.  
   - Minimal fix: add backend push channel (`/ws/import-jobs/:id` or SSE) and switch client to subscription with fallback polling.

2. **HIGH — Requirement gap: rent-change conflict workflow is binary; no explicit counterproposal negotiation state/data model**  
   - Evidence: schema has single `status` field and service only handles `pending -> approved/rejected` paths (`backend/migrations/00000000000000_initial/up.sql:118-127`, `backend/src/service/lodgings.rs:236-240`, `backend/src/service/lodgings.rs:281-285`).  
   - Impact: if acceptance requires counterproposal/negotiation loop, current model cannot represent it.  
   - Minimal fix: extend model with counterproposal entity/state transitions and reviewer/publisher iteration endpoints.

3. **MEDIUM — Operational metadata exposure on unauthenticated health endpoint**  
   - Evidence: `/api/health` route is public and response includes version/profile/disk usage (`backend/src/api/mod.rs:18`, `backend/src/api/health.rs:18-25`).  
   - Impact: environment fingerprinting for unauthenticated callers.  
   - Minimal fix: split into liveness (public minimal) and readiness/details (authenticated or internal network only).

4. **MEDIUM — Metrics endpoint lacks role-level authorization**  
   - Evidence: endpoint requires authenticated context but no `require_role!` restriction (`backend/src/api/metrics.rs:17-20`).  
   - Impact: any logged-in role can access operational counters.  
   - Minimal fix: restrict to `Administrator` (and optionally internal service accounts).

5. **LOW — Bootstrap password artifact written to `/tmp`**  
   - Evidence: generated initial admin password is written to `/tmp/.tourism_init_password` (`backend/src/lib.rs:63-70`).  
   - Impact: local secret exposure risk on weakly hardened hosts.  
   - Minimal fix: write once with strict permissions and immediate expiry/rotation guard; prefer explicit env secret injection.

## Final Acceptance Decision
**Conditional acceptance (not full acceptance).**

The system is structurally strong with broad static evidence for RBAC, facility scoping, inventory integrity, resumable imports, and extensive test intent. However, acceptance should remain conditional until the two **HIGH** requirement gaps are resolved (real-time push channel expectation and explicit rent-change conflict/counterproposal workflow, if mandated by your target business prompt).

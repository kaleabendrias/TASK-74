# Regional Tourism Resource & Lodging Operations Portal

## 1. Document Purpose
This design document describes the implemented architecture, module boundaries, data model, security controls, and major business flows for the Regional Tourism Resource & Lodging Operations Portal.

The document is aligned to the current codebase under repo and is intended to support delivery review, onboarding, and maintenance planning.

## 2. Business Context and Scope
The system supports destination marketing organizations and affiliated clinics that need to:
- Publish and govern tourism resources.
- Manage lodging lifecycle and rent/deposit governance.
- Track facility-bound inventory, reservations, and transactions.
- Run offline-first import/export and local connector ingestion.
- Enforce role-based access and facility data scope.

Primary user roles:
- Administrator
- Publisher
- Reviewer
- Clinician
- Inventory Clerk

## 3. Runtime Architecture
### 3.1 High-Level Components
- Frontend: Rust + Yew SPA (WASM) with role-aware navigation and form workflows.
- Backend: Rust + Actix-web REST API with middleware-based auth/RBAC and service/repository layering.
- Database: PostgreSQL with Diesel schema/migrations.
- Background workers:
  - Import job runner (queued/running/retry states).
  - Scheduled resource publisher.
- Optional local MQ connector:
  - TCP listener or AMQP consumer for signed on-prem messages.

### 3.2 Boot Sequence
Backend startup performs:
1. Configuration load and secret validation.
2. DB pool initialization.
3. Migration execution.
4. Default seed data/user creation when DB is empty.
5. Background worker startup (import runner + scheduler).
6. Optional MQ consumer startup.
7. HTTPS server bind (TLS is mandatory in current runtime).

### 3.3 API Surface Organization
API routes are mounted under /api and grouped by domains:
- Health/metrics
- Auth/MFA
- Resources
- Lodgings and rent negotiation
- Inventory
- Media
- Import/export
- Connector inbound integration
- Configuration center

## 4. Backend Module Decomposition
### 4.1 API Layer
Responsibility:
- HTTP transport handling.
- Request extraction and response shaping.
- Route-level role enforcement.
- Object/facility checks before service invocation.

### 4.2 Middleware Layer
Responsibility:
- Session extraction (cookie or Bearer fallback).
- CSRF checks for mutating methods.
- RBAC context hydration.
- Facility scope derivation for constrained roles.

### 4.3 Service Layer
Responsibility:
- Business rules and deterministic validations.
- State transitions.
- Transactional updates (for coupled writes).
- Domain-specific workflows (e.g., rent negotiation, exports).

### 4.4 Repository Layer
Responsibility:
- Diesel query execution.
- SQL-backed persistence for entities and projections.
- Job and connector persistence primitives.

### 4.5 Jobs Layer
Responsibility:
- Poll queued import jobs.
- Handle retries and stale-running recovery.
- Use staging-table approach with final transactional commit.
- Publish scheduled resources when due and approved.

## 5. Frontend Design
### 5.1 Core Structure
- Router-driven multi-page SPA under authenticated shell.
- Context providers:
  - Auth context: user profile + CSRF token memory.
  - Toast context: UX notifications.
- Route guard for role-based page access.

### 5.2 UX Areas
- Login and optional MFA challenge.
- Dashboard and role-adapted sidebar menu.
- Resource management (list, create/edit, history, transitions).
- Lodging management (periods, rent negotiation actions).
- Inventory views and transaction/audit workflows.
- Import/export panel with upload and approval actions.
- Security settings page for MFA controls.

### 5.3 Media UX
- Client-side extension and size checks.
- Preview URL creation for uploaded files.
- Inline error display for invalid selections.

## 6. Data Model Overview
Key tables include:
- facilities
- users
- sessions
- csrf_tokens
- resources
- resource_versions
- lodgings
- lodging_periods
- lodging_rent_changes
- warehouses
- bins
- inventory_lots
- inventory_transactions
- media_files
- review_decisions
- import_jobs
- idempotency_keys
- api_connector_logs
- config_parameters
- audit_log
- export_approvals

Notable characteristics:
- Immutable transaction logging for inventory operations.
- Resource version snapshots before mutation.
- Import job progress/retries/failure logs stored in DB.
- Export approval state and watermark persistence.

## 7. Core Business Flows
### 7.1 Authentication and MFA
- Username/password login.
- Optional TOTP challenge if user MFA is enabled and feature flag permits.
- Session + CSRF token issuance on success.
- MFA setup/confirm/disable endpoints for authenticated users.

### 7.2 Resource Lifecycle
States:
- draft -> in_review -> published -> offline
- offline -> draft

Behavior:
- Server-side validation for title/tags/hours/pricing/geo.
- Version snapshot written on update.
- Scheduled publish timestamp supported with timezone offset input.
- Scheduled publisher transitions approved in_review resources when due.

### 7.3 Lodging Lifecycle and Rent Governance
- Lodging create/update with amenity validation.
- Deposit cap enforcement: deposit <= 1.5 x monthly rent.
- Vacancy period constraints:
  - min nights >= 7
  - max nights <= 365
  - overlap prevention
- Rent-change workflow:
  - request
  - reviewer approve/reject
  - counterproposal
  - accept counterproposal

### 7.4 Inventory Operations
- Warehouse/bin listing with facility scope checks.
- Lot creation with location integrity validation.
- Reservation against on-hand stock.
- Inbound/outbound transactions with quantity controls.
- Near-expiry flag for <= 30-day window.
- Printable audit HTML for lot transaction history.

### 7.5 Media Upload/Download
- Allowlisted file extensions and MIME sniffing.
- Size enforcement based on configured max bytes.
- SHA-256 checksum persisted.
- Facility-scoped download checks for constrained users.

### 7.6 Import/Export
Import:
- XLSX-only upload with signature validation.
- Max 10,000 rows per job.
- Job queue with retries up to max_retries.
- Staging-table load and final transactional commit.
- Failure logs and resumable cursor behavior.

Export:
- Request/approve/download flow.
- Second-person approval (requester cannot self-approve).
- Optional watermark inclusion.
- PII masking applied for common email/phone keys before export file generation.

### 7.7 Connector Inbound Integration
- Local-network inbound endpoint.
- Required headers:
  - Authorization
  - X-Nonce
  - X-Timestamp
- 5-minute replay window check.
- HMAC verification.
- Idempotency key insert-on-conflict semantics to prevent duplicate processing.

## 8. Validation and Deterministic Rule Set
Implemented server-side rules include:
- Required fields and type checks.
- Numeric bounds (geo, quantities, nights, deposit ratio).
- Allowed state transitions by role.
- Media reference existence checks.
- Location hierarchy consistency (facility -> warehouse -> bin).
- Connector signature/timestamp/nonce validation.

## 9. Security Model
### 9.1 Transport and Secrets
- HTTPS startup path with rustls and local cert/key requirement.
- Secret validation with profile-aware strictness.

### 9.2 Authentication and Session Security
- Argon2id password hashing.
- Session token hashing via HMAC.
- CSRF token issuance and mutating-request validation.

### 9.3 Authorization
- Route-level role checks for domain operations.
- Facility scope enforcement for Clinician and Inventory Clerk contexts.
- Admin-only controls for metrics/config center.

### 9.4 Data Protection
- AES-GCM encryption for sensitive contact fields at rest.
- Upload type and checksum controls.
- Export masking and optional watermarking.

## 10. Observability and Operations
- Structured JSON logging via tracing.
- Health endpoints:
  - public liveness
  - authenticated readiness
- Prometheus-style metrics endpoint (Administrator restricted).
- Config parameter API for profile-specific operational values.

## 11. Test Strategy (Static Summary)
Test crates:
- unit_tests: domain/crypto/validation-focused tests.
- API_tests: endpoint and integration-flow coverage.

Current static test suite composition covers:
- Auth, MFA, RBAC, CRUD, state machines
- Inventory, media, import/export, connector envelope
- Metrics/health and config controls

## 12. Known Implementation Notes
- Facility assignment semantics for some content entities allow null facility, and constrained-role access checks may reject null-assigned entities.
- Screen-side masking helpers exist in frontend utilities; consumption should be verified in view-level rendering paths.
- Canary and maintenance config fields are present and modeled; operational runtime behavior should be validated against release policy expectations.

## 13. Future Extension Directions
- Enforce stricter facility assignment policy for scoped content entities.
- Expand dictionary and maintenance policy behavior into active request gating.
- Add richer audit querying and retention policy controls.
- Expand export field-level masking policy map and test matrix.
- Add endpoint versioning and deprecation strategy.

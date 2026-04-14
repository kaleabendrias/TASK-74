# API Specification

## 1. Overview
This API specification documents the currently implemented backend endpoints for the Regional Tourism Resource & Lodging Operations Portal.

- Base path: /api
- Protocol: HTTPS (TLS-enabled deployment expected)
- Content type: application/json (except multipart uploads and specific stream/download responses)

## 2. Authentication and Authorization
### 2.1 Session Model
Authentication is session-based with CSRF protection:
- Login issues session token and CSRF token.
- Session token can be sent via cookie or Authorization: Bearer token.
- Mutating requests require X-CSRF-Token except explicitly exempt paths.

### 2.2 Roles
- Administrator
- Publisher
- Reviewer
- Clinician
- InventoryClerk

### 2.3 Error Shape
Typical error payload:
- code: stable machine code
- message: human-readable summary
- details: optional field-level errors

## 3. Endpoint Catalog

## 3.1 Health and Metrics
### GET /api/health
Purpose:
- Public liveness probe.
Auth:
- None.
Response:
- 200 with minimal status object.

### GET /api/health/ready
Purpose:
- Authenticated readiness details.
Auth:
- Required.
Response:
- 200 with service/version/uptime/db/disk/profile fields.

### GET /api/metrics
Purpose:
- Prometheus-compatible operational metrics.
Auth:
- Administrator.
Response:
- 200 text/plain metrics format.

## 3.2 Authentication and MFA
### POST /api/auth/login
Purpose:
- Authenticate user and issue session/CSRF.
Request body:
- username: string
- password: string
- totp_code: optional string
Behavior:
- Returns MFA challenge shape when MFA code is required.

### POST /api/auth/logout
Purpose:
- Invalidate active session.
Auth:
- Required.

### GET /api/auth/me
Purpose:
- Fetch current user profile.
Auth:
- Required.

### GET /api/auth/mfa/setup
Purpose:
- Generate TOTP setup payload.
Auth:
- Required.

### POST /api/auth/mfa/confirm
Purpose:
- Confirm TOTP and enable MFA.
Auth:
- Required.
Request body:
- secret_base64
- code

### POST /api/auth/mfa/disable
Purpose:
- Disable MFA after code verification.
Auth:
- Required.
Request body:
- code

## 3.3 Resources
### POST /api/resources
Purpose:
- Create resource record.
Auth:
- Administrator, Publisher.
Request body (major fields):
- title
- category
- tags
- hours
- pricing
- address
- latitude, longitude
- media_refs
- scheduled_publish_at
- tz_offset_minutes
- contact_info

### GET /api/resources
Purpose:
- List resources with filters and pagination.
Auth:
- Administrator, Publisher, Reviewer, Clinician.
Query:
- page, per_page
- state
- category
- tag
- search
- sort_by, sort_order

### GET /api/resources/{id}
Purpose:
- Get one resource.
Auth:
- Administrator, Publisher, Reviewer, Clinician.

### PUT /api/resources/{id}
Purpose:
- Update resource and/or transition state.
Auth:
- Administrator, Publisher, Reviewer.
Notes:
- Reviewer edits are state-restricted.
- Version snapshot is written before mutation.

### GET /api/resources/{id}/versions
Purpose:
- Retrieve version history for a resource.
Auth:
- Administrator, Publisher, Reviewer.

## 3.4 Lodgings
### POST /api/lodgings
Purpose:
- Create lodging.
Auth:
- Administrator, Publisher.

### GET /api/lodgings
Purpose:
- List lodgings.
Auth:
- Administrator, Publisher, Reviewer, Clinician.

### GET /api/lodgings/{id}
Purpose:
- Get one lodging.
Auth:
- Administrator, Publisher, Reviewer, Clinician.

### PUT /api/lodgings/{id}
Purpose:
- Update lodging and/or transition state.
Auth:
- Administrator, Publisher, Reviewer.

### GET /api/lodgings/{id}/periods
Purpose:
- List lodging periods.
Auth:
- Administrator, Publisher, Reviewer, Clinician.

### PUT /api/lodgings/{id}/periods
Purpose:
- Create/replace lodging period.
Auth:
- Administrator, Publisher.
Rules:
- min_nights >= 7
- max_nights <= 365
- no overlap

### PUT /api/lodgings/{id}/rent-change
Purpose:
- Submit rent/deposit change request.
Auth:
- Administrator, Publisher.
Rules:
- deposit cap validation enforced.

### GET /api/lodgings/rent-changes/pending
Purpose:
- List pending/countered rent changes for review.
Auth:
- Administrator, Reviewer.

### POST /api/lodgings/{id}/rent-change/{change_id}/approve
Purpose:
- Approve rent change and apply values.
Auth:
- Administrator, Reviewer.

### POST /api/lodgings/{id}/rent-change/{change_id}/reject
Purpose:
- Reject pending rent change.
Auth:
- Administrator, Reviewer.

### POST /api/lodgings/{id}/rent-change/{change_id}/counterpropose
Purpose:
- Store reviewer counterproposal.
Auth:
- Administrator, Reviewer.

### POST /api/lodgings/{id}/rent-change/{change_id}/accept-counter
Purpose:
- Accept reviewer counterproposal and apply values.
Auth:
- Administrator, Publisher.

## 3.5 Inventory
### GET /api/inventory/warehouses
Purpose:
- List warehouses, optionally by facility.
Auth:
- Administrator, InventoryClerk, Clinician.

### GET /api/inventory/bins
Purpose:
- List bins by warehouse_id.
Auth:
- Administrator, InventoryClerk, Clinician.

### POST /api/inventory/lots
Purpose:
- Create inventory lot.
Auth:
- Administrator, InventoryClerk.

### GET /api/inventory/lots
Purpose:
- List lots, with near_expiry filter.
Auth:
- Administrator, InventoryClerk, Clinician.
Query:
- facility_id (admin use)
- near_expiry (bool)

### GET /api/inventory/lots/{id}
Purpose:
- Get one lot.
Auth:
- Administrator, InventoryClerk, Clinician.

### POST /api/inventory/lots/{id}/reserve
Purpose:
- Reserve quantity from lot.
Auth:
- Administrator, InventoryClerk.

### POST /api/inventory/transactions
Purpose:
- Create inbound/outbound transaction.
Auth:
- Administrator, InventoryClerk.

### GET /api/inventory/transactions
Purpose:
- Query transaction history.
Auth:
- Administrator, InventoryClerk, Clinician.
Query:
- lot_id
- direction
- performed_by
- from_date
- to_date

### GET /api/inventory/transactions/audit-print
Purpose:
- Return printable HTML audit view for lot.
Auth:
- Administrator, InventoryClerk.
Query:
- lot_id

## 3.6 Media
### POST /api/media/upload
Purpose:
- Upload and persist media metadata.
Auth:
- Administrator, Publisher.
Content type:
- multipart/form-data with file field.
Validation:
- Allowlisted types + sniff check + checksum + size limit.

### GET /api/media/{id}/download
Purpose:
- Retrieve media file.
Auth:
- Required.

## 3.7 Import and Export
### POST /api/import/upload
Purpose:
- Upload XLSX and enqueue import job.
Auth:
- Administrator, InventoryClerk.
Content type:
- multipart/form-data.
Rules:
- XLSX signature required.
- max 50 MB upload constraint.

### GET /api/import/jobs/{id}
Purpose:
- Poll import job status/progress.
Auth:
- Job owner or Administrator.

### GET /api/import/jobs/{id}/stream
Purpose:
- SSE stream of job updates.
Auth:
- Job owner or Administrator.
Response type:
- text/event-stream

### POST /api/export/request
Purpose:
- Create export approval request.
Auth:
- Administrator, Reviewer.

### POST /api/export/approve/{id}
Purpose:
- Approve export request.
Auth:
- Administrator, Reviewer.
Rules:
- requester cannot self-approve.

### GET /api/export/download/{id}
Purpose:
- Download approved export workbook.
Auth:
- Requester or Administrator.
Response type:
- application/vnd.openxmlformats-officedocument.spreadsheetml.sheet

### GET /api/export/pending
Purpose:
- List pending export approvals.
Auth:
- Administrator, Reviewer.

## 3.8 Connector
### POST /api/connector/inbound
Purpose:
- Accept signed local integration payload.
Auth:
- Signature-based endpoint security.
Required headers:
- Authorization
- X-Nonce
- X-Timestamp
Controls:
- 5-minute anti-replay window.
- Idempotency key conflict protection.

## 3.9 Configuration Center
### GET /api/config
Purpose:
- List config parameters for current profile.
Auth:
- Administrator.

### POST /api/config
Purpose:
- Create/update config key.
Auth:
- Administrator.

### GET /api/config/{key}
Purpose:
- Get one config value for current profile.
Auth:
- Administrator.

## 4. Request and Response Notes
### 4.1 Common Success Codes
- 200 OK
- 201 Created

### 4.2 Common Error Codes
- 400 BAD_REQUEST
- 401 UNAUTHORIZED
- 403 FORBIDDEN
- 404 NOT_FOUND
- 409 CONFLICT
- 413 PAYLOAD_TOO_LARGE
- 422 VALIDATION_ERROR / domain-specific unprocessable codes
- 500 INTERNAL_ERROR

## 5. Domain Rule Highlights
- Resource and lodging workflows enforce role-bound state transitions.
- Deposit cap rule: deposit <= 1.5 x monthly rent.
- Lodging period windows enforce 7..365 night limits.
- Import jobs cap at 10,000 rows and support retries and resumable cursor behavior.
- Export approval enforces second-person control.

## 6. Implementation Notes
- API namespace is stable under /api.
- No delete endpoints are currently exposed for major entities.
- Some DTO fields are accepted for write workflows but may not be returned in all read responses.

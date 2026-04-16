# Test Coverage Audit

## Project Type Detection
- Declared at README top: "full-stack web application".
- Strict normalization result: inferred project type = **fullstack**.
- Note: exact token `fullstack` is not present verbatim; wording uses "full-stack web".

## Backend Endpoint Inventory
Source: `repo/backend/src/api/mod.rs` via `/api` scope.

1. GET /api/health
2. GET /api/health/ready
3. POST /api/auth/login
4. POST /api/auth/logout
5. GET /api/auth/me
6. GET /api/auth/mfa/setup
7. POST /api/auth/mfa/confirm
8. POST /api/auth/mfa/disable
9. POST /api/resources
10. GET /api/resources
11. GET /api/resources/{id}
12. PUT /api/resources/{id}
13. GET /api/resources/{id}/versions
14. POST /api/lodgings
15. GET /api/lodgings
16. GET /api/lodgings/{id}
17. PUT /api/lodgings/{id}
18. GET /api/lodgings/{id}/periods
19. PUT /api/lodgings/{id}/periods
20. PUT /api/lodgings/{id}/rent-change
21. GET /api/lodgings/rent-changes/pending
22. POST /api/lodgings/{id}/rent-change/{change_id}/approve
23. POST /api/lodgings/{id}/rent-change/{change_id}/reject
24. POST /api/lodgings/{id}/rent-change/{change_id}/counterpropose
25. POST /api/lodgings/{id}/rent-change/{change_id}/accept-counter
26. GET /api/inventory/warehouses
27. GET /api/inventory/bins
28. POST /api/inventory/lots
29. GET /api/inventory/lots
30. GET /api/inventory/lots/{id}
31. POST /api/inventory/lots/{id}/reserve
32. POST /api/inventory/transactions
33. GET /api/inventory/transactions
34. GET /api/inventory/transactions/audit-print
35. POST /api/media/upload
36. GET /api/media/{id}/download
37. POST /api/import/upload
38. GET /api/import/jobs/{id}
39. GET /api/import/jobs/{id}/stream
40. POST /api/export/request
41. POST /api/export/approve/{id}
42. GET /api/export/download/{id}
43. GET /api/export/pending
44. POST /api/connector/inbound
45. GET /api/config
46. POST /api/config
47. GET /api/config/{key}
48. GET /api/metrics

Total endpoints: **48**

## API Test Mapping Table
Coverage criterion applied: exact method + path request evidence in tests.

| Endpoint | Covered | Test type | Test files | Evidence (function refs) |
|---|---|---|---|---|
| GET /api/health | yes | true no-mock HTTP | API_tests/src/test_metrics_health.rs | health_liveness_is_public_and_returns_200; health_liveness_does_not_expose_internal_fields |
| GET /api/health/ready | yes | true no-mock HTTP | API_tests/src/test_metrics_health.rs | health_readiness_requires_authentication; health_readiness_returns_full_details_when_authenticated |
| POST /api/auth/login | yes | true no-mock HTTP | API_tests/src/test_auth.rs; API_tests/src/helpers.rs | login_success; helper login_as |
| POST /api/auth/logout | yes | true no-mock HTTP | API_tests/src/test_auth.rs; API_tests/src/test_state_machines.rs | logout_clears_session; auth_logout_invalidates_session |
| GET /api/auth/me | yes | true no-mock HTTP | API_tests/src/test_auth.rs; API_tests/src/test_state_machines.rs | me_returns_profile; auth_me_returns_user_profile |
| GET /api/auth/mfa/setup | yes | true no-mock HTTP | API_tests/src/test_user_management.rs | mfa_setup_returns_totp_provisioning_payload |
| POST /api/auth/mfa/confirm | yes | true no-mock HTTP | API_tests/src/test_user_management.rs | mfa_confirm_missing_secret_returns_400; mfa_confirm_invalid_code_returns_401 |
| POST /api/auth/mfa/disable | yes | true no-mock HTTP | API_tests/src/test_user_management.rs | mfa_disable_when_not_enabled_returns_400 |
| POST /api/resources | yes | true no-mock HTTP | API_tests/src/test_resources.rs | create_resource_valid |
| GET /api/resources | yes | true no-mock HTTP | API_tests/src/test_resources.rs; API_tests/src/test_response_schemas.rs | list_resources_paginated; unauthenticated_request_returns_401_with_schema |
| GET /api/resources/{id} | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs | resource_get_by_id_returns_resource |
| PUT /api/resources/{id} | yes | true no-mock HTTP | API_tests/src/test_resources.rs; API_tests/src/test_crud_coverage.rs | state_transition_full_lifecycle; resource_put_updates_fields_without_state_change |
| GET /api/resources/{id}/versions | yes | true no-mock HTTP | API_tests/src/test_state_machines.rs; API_tests/src/test_security.rs | resource_put_increments_version_and_creates_version_record; resource_versions_history |
| POST /api/lodgings | yes | true no-mock HTTP | API_tests/src/test_lodgings.rs | create_lodging_valid |
| GET /api/lodgings | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs | lodging_list_returns_array |
| GET /api/lodgings/{id} | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs; API_tests/src/test_rent_negotiation.rs | lodging_get_by_id_returns_lodging |
| PUT /api/lodgings/{id} | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs | lodging_put_updates_fields |
| GET /api/lodgings/{id}/periods | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs | lodging_periods_list_returns_array |
| PUT /api/lodgings/{id}/periods | yes | true no-mock HTTP | API_tests/src/test_lodgings.rs; API_tests/src/test_crud_coverage.rs | vacancy_period_7_nights_ok |
| PUT /api/lodgings/{id}/rent-change | yes | true no-mock HTTP | API_tests/src/test_lodgings.rs; API_tests/src/test_rent_negotiation.rs | rent_change_approve_lifecycle; rent_change_full_negotiation_flow |
| GET /api/lodgings/rent-changes/pending | yes | true no-mock HTTP | API_tests/src/test_rent_negotiation.rs | rent_change_full_negotiation_flow |
| POST /api/lodgings/{id}/rent-change/{change_id}/approve | yes | true no-mock HTTP | API_tests/src/test_lodgings.rs; API_tests/src/test_rent_negotiation.rs | rent_change_approve_lifecycle |
| POST /api/lodgings/{id}/rent-change/{change_id}/reject | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs | rent_change_reject_lifecycle |
| POST /api/lodgings/{id}/rent-change/{change_id}/counterpropose | yes | true no-mock HTTP | API_tests/src/test_rent_negotiation.rs | rent_change_full_negotiation_flow |
| POST /api/lodgings/{id}/rent-change/{change_id}/accept-counter | yes | true no-mock HTTP | API_tests/src/test_rent_negotiation.rs | rent_change_full_negotiation_flow |
| GET /api/inventory/warehouses | yes | true no-mock HTTP | API_tests/src/test_state_machines.rs | inventory_list_warehouses_returns_json_array |
| GET /api/inventory/bins | yes | true no-mock HTTP | API_tests/src/test_inventory.rs; API_tests/src/test_state_machines.rs | clerk_cannot_list_bins_from_other_facility |
| POST /api/inventory/lots | yes | true no-mock HTTP | API_tests/src/test_inventory.rs | create_lot_and_list |
| GET /api/inventory/lots | yes | true no-mock HTTP | API_tests/src/test_inventory.rs | create_lot_and_list |
| GET /api/inventory/lots/{id} | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs; API_tests/src/test_security.rs | inventory_lot_get_by_id_returns_lot |
| POST /api/inventory/lots/{id}/reserve | yes | true no-mock HTTP | API_tests/src/test_inventory.rs | reserve_success; over_reservation_returns_409 |
| POST /api/inventory/transactions | yes | true no-mock HTTP | API_tests/src/test_inventory.rs | transaction_recorded |
| GET /api/inventory/transactions | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs | inventory_transaction_list_returns_array |
| GET /api/inventory/transactions/audit-print | yes | true no-mock HTTP | API_tests/src/test_inventory.rs; API_tests/src/test_state_machines.rs | audit_print_returns_html |
| POST /api/media/upload | yes | true no-mock HTTP | API_tests/src/test_media.rs; API_tests/src/test_crud_coverage.rs | upload_valid_png |
| GET /api/media/{id}/download | yes | true no-mock HTTP | API_tests/src/test_media.rs; API_tests/src/test_crud_coverage.rs | download_uploaded_file |
| POST /api/import/upload | yes | true no-mock HTTP | API_tests/src/test_import_export.rs; API_tests/src/test_crud_coverage.rs | import_xlsx_only; import_upload_valid_xlsx_creates_job |
| GET /api/import/jobs/{id} | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs; API_tests/src/test_job_recovery.rs | import_upload_valid_xlsx_creates_job; crashed_job_remains_visible_via_api |
| GET /api/import/jobs/{id}/stream | yes | true no-mock HTTP | API_tests/src/test_crud_coverage.rs | import_job_sse_stream_endpoint_responds |
| POST /api/export/request | yes | true no-mock HTTP | API_tests/src/test_import_export.rs; API_tests/src/test_user_management.rs | export_request_and_approve_flow |
| POST /api/export/approve/{id} | yes | true no-mock HTTP | API_tests/src/test_import_export.rs; API_tests/src/test_user_management.rs | export_request_and_approve_flow; export_requester_cannot_approve_own_export |
| GET /api/export/download/{id} | yes | true no-mock HTTP | API_tests/src/test_import_export.rs; API_tests/src/test_security.rs | export_request_and_approve_flow |
| GET /api/export/pending | yes | true no-mock HTTP | API_tests/src/test_user_management.rs | export_pending_list_returns_array |
| POST /api/connector/inbound | yes | true no-mock HTTP | API_tests/src/test_connector.rs; API_tests/src/test_connector_envelope.rs | connector_valid_payload; envelope_accepted_with_valid_fields |
| GET /api/config | yes | true no-mock HTTP | API_tests/src/test_config.rs | config_list_returns_array_for_admin |
| POST /api/config | yes | true no-mock HTTP | API_tests/src/test_config.rs | config_upsert_creates_new_parameter |
| GET /api/config/{key} | yes | true no-mock HTTP | API_tests/src/test_config.rs | config_get_by_key_returns_parameter |
| GET /api/metrics | yes | true no-mock HTTP | API_tests/src/test_metrics_health.rs | metrics_returns_prometheus_format_for_admin |

## API Test Classification
1. True No-Mock HTTP
- API_tests/src/test_auth.rs
- API_tests/src/test_config.rs
- API_tests/src/test_connector.rs
- API_tests/src/test_connector_envelope.rs
- API_tests/src/test_crud_coverage.rs
- API_tests/src/test_import_export.rs
- API_tests/src/test_inventory.rs
- API_tests/src/test_lodgings.rs
- API_tests/src/test_media.rs
- API_tests/src/test_metrics_health.rs
- API_tests/src/test_rbac.rs
- API_tests/src/test_rent_negotiation.rs
- API_tests/src/test_resources.rs
- API_tests/src/test_response_schemas.rs
- API_tests/src/test_security.rs
- API_tests/src/test_state_machines.rs
- API_tests/src/test_user_management.rs
- API_tests/src/helpers.rs (reqwest HTTP login helper)

2. HTTP with Mocking
- None detected.

3. Non-HTTP (unit/integration without HTTP)
- API_tests/src/test_job_recovery.rs (direct DB insert + repository function calls; one endpoint check included)
- repo/unit_tests/src/* (backend logic unit tests)
- repo/frontend_tests/src/* (frontend logic unit tests)

## Mock Detection
Static scan for mock/stub patterns (`jest.mock`, `vi.mock`, `sinon.stub`, `mockall`, `wiremock`, Actix in-process test harness) found no evidence in API_tests.

Detected literal "fake" strings are test payload names only:
- API_tests/src/test_media.rs (`%PDF-1.4 fake content`)
- API_tests/src/test_crud_coverage.rs (`fake.png`)

## Coverage Summary
- Total endpoints: 48
- Endpoints with HTTP tests: 48
- Endpoints with TRUE no-mock HTTP tests: 48
- HTTP coverage: 100%
- True API coverage: 100%

## Unit Test Summary

### Backend Unit Tests
Test files (module-level):
- unit_tests/src/test_argon2id.rs
- unit_tests/src/test_aes_gcm.rs
- unit_tests/src/test_totp.rs
- unit_tests/src/test_hmac.rs
- unit_tests/src/test_csrf_sha256.rs
- unit_tests/src/test_validation.rs
- unit_tests/src/test_state_machine.rs
- unit_tests/src/test_deposit_cap.rs
- unit_tests/src/test_periods.rs
- unit_tests/src/test_inventory_logic.rs
- unit_tests/src/test_masking.rs
- unit_tests/src/test_near_expiry.rs
- unit_tests/src/test_night_bounds.rs
- unit_tests/src/test_import_validation.rs
- unit_tests/src/test_tz_scheduling.rs
- unit_tests/src/test_export_watermark.rs
- unit_tests/src/test_import_resumable.rs
- unit_tests/src/test_rbac_permissions.rs

Backend modules covered (evidence by imports):
- Crypto: `tourism_backend::crypto::*`
- Service logic: `service::validation`, `service::resources`, `service::inventory`, `service::masking`, `service::import_export`
- Middleware authz logic: `middleware::rbac::has_permission`
- Jobs runner helpers: `jobs::runner::*`
- Repository import jobs: `repository::import_jobs`

Important backend modules NOT unit-tested directly:
- API handlers in backend/src/api/*.rs
- Most repository modules (`resources`, `lodgings`, `media`, `users`, `sessions`, `config`, `connector`, `inventory`, `export`, `audit`) as isolated unit targets
- middleware/auth_guard.rs
- Service modules with external I/O heavy paths (auth, connector, media, mq_connector)

### Frontend Unit Tests (STRICT REQUIREMENT)
Frontend unit tests: **PRESENT**

Detection rule checks:
- Identifiable frontend test files exist: yes (`frontend_tests/src/test_*.rs`)
- Tests target frontend logic/components: yes (imports from `frontend_logic` production crate)
- Framework/tool evident: yes (Rust built-in test framework via `#[test]` in `frontend_tests` crate)
- Tests import actual frontend modules: yes (`frontend_logic::{auth,toast,sidebar,routing,validation,mask,models,app_shell}`)

Frontend test files:
- frontend_tests/src/test_auth_context.rs
- frontend_tests/src/test_toast_context.rs
- frontend_tests/src/test_sidebar_visibility.rs
- frontend_tests/src/test_route_permissions.rs
- frontend_tests/src/test_models_serialization.rs
- frontend_tests/src/test_pii_masking.rs
- frontend_tests/src/test_client_validation.rs
- frontend_tests/src/test_workflow_scenarios.rs
- frontend_tests/src/test_component_behaviors.rs

Frameworks/tools detected:
- Rust built-in test harness (`#[test]`)
- Cargo workspace package `frontend_tests`

Frontend components/modules covered:
- `frontend_logic::auth`
- `frontend_logic::toast`
- `frontend_logic::sidebar`
- `frontend_logic::routing`
- `frontend_logic::validation`
- `frontend_logic::mask`
- `frontend_logic::models`
- `frontend_logic::app_shell`

Important frontend components/modules NOT unit-tested directly:
- Yew UI rendering in frontend/src/components/* (`app.rs`, `route_guard.rs`, `sidebar.rs`, `toast.rs`) is covered indirectly via extracted logic, not component rendering tests.
- Page-level integration behavior in frontend/src/pages/* is not directly unit-tested in frontend_tests.

### Cross-Layer Observation
- Backend API tests are very strong and broad.
- Frontend logic unit coverage is substantial.
- Balance is acceptable (not backend-only).

## Tests Check

### API Observability Check
- Strong for most endpoints: tests typically include explicit method/path, input payloads, and response assertions (status + body fields + headers).
- Weak spots:
  - Some tests assert mostly status code with shallow body checks (example patterns in role-blocking tests).
  - Limited explicit negative-path payload assertions for a few endpoints (e.g., some logout/permission edges).

### Test Quality & Sufficiency
- Success paths: strong
- Failure paths: strong (validation, auth, RBAC, CSRF, role restrictions)
- Edge cases: good (state machines, rent negotiation, SSE stream, envelope security)
- Integration boundaries: good (real HTTP + real DB)
- Assertion depth: generally meaningful; not superficial overall

### run_tests.sh Check
- Docker-based orchestration: **OK**
- Local dependency requirement: **No hard violation detected** (uses Docker Compose for build/run and test execution)

### End-to-End Expectations (fullstack)
- Fullstack FE↔BE E2E suite exists (`repo/e2e` with Playwright specs).
- This partially compensates for lack of direct Yew DOM/unit rendering tests.

## Test Coverage Score (0-100)
**92/100**

## Score Rationale
- + Full endpoint HTTP coverage with real requests and no mocks.
- + Strong security/validation/state-machine test breadth.
- + Backend + frontend unit suites present.
- - Some endpoint tests are shallow on response contract depth in negative paths.
- - Component rendering itself is mostly indirectly tested via extracted frontend_logic.
- - One API test file (`test_job_recovery.rs`) is mixed-mode (mostly non-HTTP internals), which is valid but not endpoint coverage.

## Key Gaps
1. Direct Yew component rendering tests are sparse/absent; logic-level tests dominate.
2. Some RBAC/negative tests prioritize status-only assertions over detailed error schema verification.
3. Backend unit tests do not directly target many repository modules and handler-level pure functions.

## Confidence & Assumptions
- Confidence: high
- Assumptions:
  - Endpoint inventory is exclusively from `backend/src/api/mod.rs` route registration.
  - Coverage classification is static and based on visible request calls in source.
  - Runtime behavior is not inferred beyond static wiring and scripts.

## Test Coverage Verdict
**PASS (with quality caveats)**

---

# README Audit

README location checked: `repo/README.md` (exists)

## Hard Gate Evaluation

### Formatting
- Pass: clean markdown, coherent sections, readable hierarchy.

### Startup Instructions (fullstack requirement)
- Pass: includes `docker-compose up --build -d`.

### Access Method
- Pass: explicit frontend and backend URLs with ports.

### Verification Method
- **Fail (hard-gate strictness):** README does not provide explicit verification procedure/flow (e.g., concrete curl/Postman checks or step-by-step UI workflow assertions). It lists endpoints/URLs but not a deterministic "how to confirm working" checklist.

### Environment Rules (Docker-contained)
- Pass: no npm/pip/apt/manual DB setup instructions.

### Demo Credentials (auth present)
- Pass: includes username + password for all listed roles.

## High Priority Issues
1. Missing strict verification runbook in README (hard-gate failure): no explicit validation flow for API and UI behavior beyond URL listing.
2. Top-level project type token is not normalized to required strict set (`fullstack` expected token); wording uses "full-stack web".

## Medium Priority Issues
1. Testing section references running `chmod +x` and `./run_tests.sh` but does not state expected key pass criteria beyond exit code (no sample checks).
2. Architecture section is concise but lacks explicit data-flow/workflow diagrams or request lifecycle summary.

## Low Priority Issues
1. README could include troubleshooting snippets (container health, TLS cert mismatch hints).
2. README could include explicit role-to-feature matrix to mirror RBAC scope in tests.

## Hard Gate Failures
1. Verification method missing deterministic validation steps.

## README Verdict
**PARTIAL PASS**

## README Compliance Verdict
Given strict-mode hard gates, README is **not full PASS** due to verification-method gap.

---

# Final Verdicts
1. Test Coverage Audit Verdict: **PASS (with quality caveats)**
2. README Audit Verdict: **PARTIAL PASS (hard-gate verification failure)**

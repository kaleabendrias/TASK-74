# Test Coverage Audit

## Scope and Method
- Mode: static inspection only.
- No runtime execution, no test runs, no package/container commands executed for this audit.
- Primary evidence sources:
  - Endpoint registry: `repo/backend/src/api/mod.rs` (`configure_routes`)
  - API tests: `repo/API_tests/src/*.rs`
  - Unit/frontend tests: `repo/unit_tests/src/*`, `repo/frontend_tests/src/*`
  - Test runner constraints: `repo/run_tests.sh`

## Backend Endpoint Inventory
Source of truth: `repo/backend/src/api/mod.rs` (`configure_routes`).

1. GET `/api/health`
2. GET `/api/health/ready`
3. POST `/api/auth/login`
4. POST `/api/auth/logout`
5. GET `/api/auth/me`
6. GET `/api/auth/mfa/setup`
7. POST `/api/auth/mfa/confirm`
8. POST `/api/auth/mfa/disable`
9. POST `/api/resources`
10. GET `/api/resources`
11. GET `/api/resources/{id}`
12. PUT `/api/resources/{id}`
13. GET `/api/resources/{id}/versions`
14. POST `/api/lodgings`
15. GET `/api/lodgings`
16. GET `/api/lodgings/{id}`
17. PUT `/api/lodgings/{id}`
18. GET `/api/lodgings/{id}/periods`
19. PUT `/api/lodgings/{id}/periods`
20. PUT `/api/lodgings/{id}/rent-change`
21. GET `/api/lodgings/rent-changes/pending`
22. POST `/api/lodgings/{id}/rent-change/{change_id}/approve`
23. POST `/api/lodgings/{id}/rent-change/{change_id}/reject`
24. POST `/api/lodgings/{id}/rent-change/{change_id}/counterpropose`
25. POST `/api/lodgings/{id}/rent-change/{change_id}/accept-counter`
26. GET `/api/inventory/warehouses`
27. GET `/api/inventory/bins`
28. POST `/api/inventory/lots`
29. GET `/api/inventory/lots`
30. GET `/api/inventory/lots/{id}`
31. POST `/api/inventory/lots/{id}/reserve`
32. POST `/api/inventory/transactions`
33. GET `/api/inventory/transactions`
34. GET `/api/inventory/transactions/audit-print`
35. POST `/api/media/upload`
36. GET `/api/media/{id}/download`
37. POST `/api/import/upload`
38. GET `/api/import/jobs/{id}`
39. GET `/api/import/jobs/{id}/stream`
40. POST `/api/export/request`
41. POST `/api/export/approve/{id}`
42. GET `/api/export/download/{id}`
43. GET `/api/export/pending`
44. POST `/api/connector/inbound`
45. GET `/api/config`
46. POST `/api/config`
47. GET `/api/config/{key}`
48. GET `/api/metrics`

## API Test Mapping Table
Legend:
- Test type values:
  - true no-mock HTTP
  - HTTP with mocking
  - unit-only / indirect

| Endpoint | Covered | Test type | Test files | Evidence (file + test fn) |
|---|---|---|---|---|
| GET `/api/health` | yes | true no-mock HTTP | `API_tests/src/test_metrics_health.rs` | `health_liveness_is_public_and_returns_200` |
| GET `/api/health/ready` | yes | true no-mock HTTP | `API_tests/src/test_metrics_health.rs` | `health_readiness_returns_full_details_when_authenticated` |
| POST `/api/auth/login` | yes | true no-mock HTTP | `API_tests/src/test_auth.rs` | `login_success` |
| POST `/api/auth/logout` | yes | true no-mock HTTP | `API_tests/src/test_auth.rs`, `API_tests/src/test_state_machines.rs` | `logout_clears_session`, `auth_logout_invalidates_session` |
| GET `/api/auth/me` | yes | true no-mock HTTP | `API_tests/src/test_auth.rs`, `API_tests/src/test_state_machines.rs` | `me_returns_profile`, `auth_me_returns_user_profile` |
| GET `/api/auth/mfa/setup` | yes | true no-mock HTTP | `API_tests/src/test_user_management.rs` | `mfa_setup_returns_totp_provisioning_payload` |
| POST `/api/auth/mfa/confirm` | yes | true no-mock HTTP | `API_tests/src/test_user_management.rs` | `mfa_confirm_missing_secret_returns_400` |
| POST `/api/auth/mfa/disable` | yes | true no-mock HTTP | `API_tests/src/test_user_management.rs` | `mfa_disable_when_not_enabled_returns_400` |
| POST `/api/resources` | yes | true no-mock HTTP | `API_tests/src/test_resources.rs`, `API_tests/src/test_security.rs` | `create_resource_valid`, `post_with_valid_csrf_succeeds` |
| GET `/api/resources` | yes | true no-mock HTTP | `API_tests/src/test_resources.rs`, `API_tests/src/test_user_management.rs` | `list_resources_paginated`, `resource_list_filtered_by_category` |
| GET `/api/resources/{id}` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs`, `API_tests/src/test_security.rs` | `resource_get_by_id_returns_resource`, `nonexistent_resource_returns_404` |
| PUT `/api/resources/{id}` | yes | true no-mock HTTP | `API_tests/src/test_resources.rs`, `API_tests/src/test_state_machines.rs` | `state_transition_full_lifecycle`, `resource_put_increments_version_and_creates_version_record` |
| GET `/api/resources/{id}/versions` | yes | true no-mock HTTP | `API_tests/src/test_security.rs`, `API_tests/src/test_state_machines.rs` | `resource_versions_returns_history`, `resource_put_increments_version_and_creates_version_record` |
| POST `/api/lodgings` | yes | true no-mock HTTP | `API_tests/src/test_lodgings.rs`, `API_tests/src/test_rbac.rs` | `create_lodging_valid`, `clinician_cannot_create_lodging` |
| GET `/api/lodgings` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs` | `lodging_list_returns_array` |
| GET `/api/lodgings/{id}` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs`, `API_tests/src/test_security.rs` | `lodging_get_by_id_returns_lodging`, `nonexistent_lodging_returns_404` |
| PUT `/api/lodgings/{id}` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs` | `lodging_put_updates_fields` |
| GET `/api/lodgings/{id}/periods` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs` | `lodging_periods_list_returns_array` |
| PUT `/api/lodgings/{id}/periods` | yes | true no-mock HTTP | `API_tests/src/test_lodgings.rs` | `vacancy_period_7_nights_ok` |
| PUT `/api/lodgings/{id}/rent-change` | yes | true no-mock HTTP | `API_tests/src/test_lodgings.rs`, `API_tests/src/test_rent_negotiation.rs` | `rent_change_approve_lifecycle`, `rent_change_full_negotiation_flow` |
| GET `/api/lodgings/rent-changes/pending` | yes | true no-mock HTTP | `API_tests/src/test_rent_negotiation.rs` | `pending_list_includes_countered_changes` |
| POST `/api/lodgings/{id}/rent-change/{change_id}/approve` | yes | true no-mock HTTP | `API_tests/src/test_lodgings.rs`, `API_tests/src/test_rent_negotiation.rs` | `rent_change_approve_lifecycle`, `counterpropose_on_already_approved_change_returns_422` |
| POST `/api/lodgings/{id}/rent-change/{change_id}/reject` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs` | `rent_change_reject_lifecycle` |
| POST `/api/lodgings/{id}/rent-change/{change_id}/counterpropose` | yes | true no-mock HTTP | `API_tests/src/test_rent_negotiation.rs` | `rent_change_full_negotiation_flow` |
| POST `/api/lodgings/{id}/rent-change/{change_id}/accept-counter` | yes | true no-mock HTTP | `API_tests/src/test_rent_negotiation.rs` | `rent_change_full_negotiation_flow` |
| GET `/api/inventory/warehouses` | yes | true no-mock HTTP | `API_tests/src/test_state_machines.rs` | `inventory_list_warehouses_returns_json_array` |
| GET `/api/inventory/bins` | yes | true no-mock HTTP | `API_tests/src/test_inventory.rs`, `API_tests/src/test_state_machines.rs` | `clerk_cannot_list_bins_from_other_facility`, `inventory_list_bins_for_warehouse` |
| POST `/api/inventory/lots` | yes | true no-mock HTTP | `API_tests/src/test_inventory.rs`, `API_tests/src/test_security.rs` | `create_lot_and_list`, `clerk_cannot_create_lot_on_other_facility` |
| GET `/api/inventory/lots` | yes | true no-mock HTTP | `API_tests/src/test_inventory.rs`, `API_tests/src/test_rbac.rs` | `create_lot_and_list`, `clinician_can_view_inventory` |
| GET `/api/inventory/lots/{id}` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs`, `API_tests/src/test_security.rs` | `inventory_lot_get_by_id_returns_lot`, `nonexistent_lot_returns_404` |
| POST `/api/inventory/lots/{id}/reserve` | yes | true no-mock HTTP | `API_tests/src/test_inventory.rs` | `reserve_success` |
| POST `/api/inventory/transactions` | yes | true no-mock HTTP | `API_tests/src/test_inventory.rs`, `API_tests/src/test_crud_coverage.rs` | `transaction_recorded`, `inventory_transaction_list_returns_array` |
| GET `/api/inventory/transactions` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs` | `inventory_transaction_list_returns_array` |
| GET `/api/inventory/transactions/audit-print` | yes | true no-mock HTTP | `API_tests/src/test_inventory.rs`, `API_tests/src/test_state_machines.rs` | `audit_print_returns_html`, `inventory_audit_print_returns_html_for_admin` |
| POST `/api/media/upload` | yes | true no-mock HTTP | `API_tests/src/test_media.rs`, `API_tests/src/test_crud_coverage.rs` | `upload_valid_png`, `media_upload_mime_mismatch_rejected` |
| GET `/api/media/{id}/download` | yes | true no-mock HTTP | `API_tests/src/test_media.rs`, `API_tests/src/test_crud_coverage.rs` | `download_uploaded_file`, `media_download_returns_file_bytes` |
| POST `/api/import/upload` | yes | true no-mock HTTP | `API_tests/src/test_import_export.rs`, `API_tests/src/test_crud_coverage.rs` | `import_xlsx_only`, `import_upload_valid_xlsx_creates_job` |
| GET `/api/import/jobs/{id}` | yes | true no-mock HTTP | `API_tests/src/test_job_recovery.rs`, `API_tests/src/test_crud_coverage.rs` | `crashed_job_remains_visible_via_api`, `import_upload_valid_xlsx_creates_job` |
| GET `/api/import/jobs/{id}/stream` | yes | true no-mock HTTP | `API_tests/src/test_crud_coverage.rs` | `import_job_sse_stream_endpoint_responds` |
| POST `/api/export/request` | yes | true no-mock HTTP | `API_tests/src/test_import_export.rs`, `API_tests/src/test_security.rs` | `export_request_and_approve_flow`, `export_download_blocked_for_non_requester_non_admin` |
| POST `/api/export/approve/{id}` | yes | true no-mock HTTP | `API_tests/src/test_import_export.rs`, `API_tests/src/test_user_management.rs` | `export_request_and_approve_flow`, `export_requester_cannot_approve_own_export` |
| GET `/api/export/download/{id}` | yes | true no-mock HTTP | `API_tests/src/test_import_export.rs`, `API_tests/src/test_security.rs` | `export_request_and_approve_flow`, `export_data_has_pii_masking` |
| GET `/api/export/pending` | yes | true no-mock HTTP | `API_tests/src/test_user_management.rs` | `export_pending_list_returns_array` |
| POST `/api/connector/inbound` | yes | true no-mock HTTP | `API_tests/src/test_connector.rs`, `API_tests/src/test_connector_envelope.rs` | `connector_valid_payload`, `atomic_idempotency_concurrent_same_nonce` |
| GET `/api/config` | yes | true no-mock HTTP | `API_tests/src/test_config.rs`, `API_tests/src/test_security.rs` | `config_list_returns_array_for_admin`, `config_endpoint_requires_admin` |
| POST `/api/config` | yes | true no-mock HTTP | `API_tests/src/test_config.rs` | `config_upsert_creates_new_parameter` |
| GET `/api/config/{key}` | yes | true no-mock HTTP | `API_tests/src/test_config.rs` | `config_get_by_key_returns_parameter` |
| GET `/api/metrics` | yes | true no-mock HTTP | `API_tests/src/test_metrics_health.rs` | `metrics_returns_prometheus_format_for_admin` |

## API Test Classification

### 1) True No-Mock HTTP
Evidence for real HTTP layer:
- Requests are made through `reqwest::Client` to `base_url()` (`API_tests/src/helpers.rs`, `base_url`, `authed_client`, `bearer_client`).
- Endpoint calls use full `/api/...` paths in test files such as:
  - `API_tests/src/test_auth.rs` (`login_success`)
  - `API_tests/src/test_resources.rs` (`create_resource_valid`)
  - `API_tests/src/test_rent_negotiation.rs` (`rent_change_full_negotiation_flow`)
- External orchestration context in `run_tests.sh` launches backend container and runs `cargo test -p api_tests` against live backend.

Classification result:
- All endpoint-mapping HTTP tests are classified `true no-mock HTTP`.

### 2) HTTP with Mocking
- None found by static inspection.

### 3) Non-HTTP (unit/integration without HTTP)
- `API_tests/src/test_job_recovery.rs`
  - `stale_running_job_reset_to_queued`
  - `recent_running_job_not_reset`
  - `exhausted_stale_job_not_reset`
  - `reset_job_is_picked_up_by_next_poll`
- These call repository functions directly (e.g., `repository::import_jobs::reset_stale_running_jobs`) and manipulate DB rows directly.

## Mock Detection Rules Audit
Patterns checked: `jest.mock`, `vi.mock`, `sinon.stub`, mock transport/service/provider overrides, direct controller bypass for HTTP tests.

Findings:
- No mocking framework usage found in `API_tests` or Rust test suites by static search.
- No explicit DI override style found in API tests.
- Direct non-HTTP repository calls exist in `API_tests/src/test_job_recovery.rs` (classified separately as non-HTTP).

## Coverage Summary
- Total endpoints: 48
- Endpoints with HTTP tests: 48
- Endpoints with true no-mock HTTP tests: 48
- HTTP coverage: 48/48 = 100.0%
- True API coverage: 48/48 = 100.0%

## Unit Test Summary
Test files / suites:
- `repo/unit_tests/src/lib.rs` includes crypto, validation, state-machine, scheduling, import/export logic suites.
- `repo/frontend_tests/src/lib.rs` includes auth/toast/sidebar/route permissions/models/masking/client-validation/workflow suites.

Modules covered (inferred from test module names + API tests):
- Controllers (HTTP handlers): covered primarily via API HTTP tests in `API_tests/src/*.rs`.
- Services: covered indirectly through API HTTP execution paths (`backend/src/service/*` exercised via endpoint tests).
- Repositories:
  - Indirectly covered via API HTTP tests.
  - Direct repository-level checks present for import recovery in `API_tests/src/test_job_recovery.rs`.
- Auth/guards/middleware:
  - Covered via auth/session/MFA/CSRF/RBAC tests (`test_auth.rs`, `test_security.rs`, `test_rbac.rs`, `test_user_management.rs`, `test_metrics_health.rs`).

Important modules not clearly unit-tested in isolation:
- `backend/src/api/*` (handler unit tests not visible; coverage is HTTP-level instead)
- Most of `backend/src/repository/*` except import job recovery direct checks
- Most of `backend/src/service/*` in isolated unit form (mainly integration-level coverage)
- `backend/src/middleware/*` appears validated via behavior tests, not explicit isolated unit modules

## API Observability Check
Assessment: mostly strong.

Evidence:
- Endpoint and method are explicit in each request call (e.g., `test_config.rs`, `test_inventory.rs`, `test_rent_negotiation.rs`).
- Request input bodies/headers/params are explicit (`json(...)`, query strings, `X-CSRF-Token`).
- Response assertions commonly include status and payload structure/content (`test_import_export.rs`, `test_metrics_health.rs`, `test_security.rs`).

Weak spots:
- Some tests assert only status code with minimal body validation (examples scattered in role/permission negative tests), reducing semantic confidence for response contract depth.

## Tests Check
Success paths:
- Extensive and present across auth/resources/lodgings/inventory/media/import-export/config/metrics.

Failure cases:
- Strong presence (auth failures, CSRF failures, RBAC denials, validation errors, 404s, conflict cases).

Edge cases:
- Present (nonce replay, signature tampering, stale job recovery, deposit caps, overlapping periods, scheduled transitions, near-expiry filters).

Validation depth:
- Good breadth in both API and unit/frontend logic tests.

Auth/permissions:
- Strongly represented (`test_rbac.rs`, `test_security.rs`, `test_metrics_health.rs`, `test_user_management.rs`).

Integration boundaries:
- API suite exercises FE-independent backend HTTP interface with DB-backed behavior.
- E2E suite exists (`repo/e2e/tests/ui_rendering.spec.ts`) for rendered UI behavior and role navigation.

Assertions quality:
- Generally meaningful (status + payload fields + side-effects), not merely pass/fail.

`run_tests.sh` check:
- Docker-based orchestration detected (`docker compose ...` across build/up/run/down). This satisfies Docker-based execution expectation.
- No mandatory local package-manager install steps inside the script; host prerequisites are Docker/Compose and curl availability.

## End-to-End Expectations (Fullstack)
- Project is fullstack (backend + frontend + e2e folders).
- E2E tests are present (`repo/e2e/tests/*.spec.ts`) and include role/login/UI workflow checks, partially covering FE↔BE behavior.
- Compensation requirement not needed because both API and E2E evidence exist.

## Test Coverage Score (0-100)
- Score: 93/100

## Score Rationale
- + Excellent endpoint coverage (100% HTTP + true no-mock mapping for declared routes).
- + Strong negative/edge/security scenarios.
- + Presence of E2E and broad unit/frontend logic suites.
- - Limited isolated unit tests for many backend service/repository/controller internals (reliance on integration coverage).
- - A subset of tests assert status-only, with less response-contract depth.

## Key Gaps
1. Isolated unit tests are sparse for many `backend/src/service/*` and `backend/src/repository/*` modules.
2. Some permission/error-path tests could strengthen response body contract assertions.
3. `backend/src/api/*` handler behavior is mostly integration-tested, not directly unit-tested.

## Confidence and Assumptions
- Confidence: high for endpoint inventory and endpoint-to-test mapping.
- Assumptions:
  - Routing source of truth is `backend/src/api/mod.rs` with no alternate route registration elsewhere.
  - `base_url()` requests hit a real running backend in intended test execution context.


# README Audit

## Project Type Detection
- README top states: "A full-stack web application..." (`repo/README.md`).
- Detected type: fullstack (explicit, not inferred fallback).

## README Location Check
- Required path exists: `repo/README.md`.

## Hard Gates

### Formatting
- PASS: markdown is clean and structured with sections/tables/code blocks.

### Startup Instructions (Backend/Fullstack)
- PASS: includes `docker-compose up --build -d` under "Running the Application".

### Access Method
- PASS: provides explicit URLs and ports:
  - frontend `https://localhost:8081`
  - backend `https://localhost:8080/api`
  - health/ready/metrics endpoints

### Verification Method
- PARTIAL/FAIL (strict):
  - README lists endpoints but does not provide a concrete verification workflow (e.g., explicit curl/Postman commands or a deterministic UI smoke-flow with expected outcomes).

### Environment Rules (Docker-contained)
- PASS:
  - No forbidden install guidance (`npm install`, `pip install`, `apt-get`, manual DB bootstrap) in README.
  - Docker/Compose prerequisite and workflow are explicit.

### Demo Credentials (Auth present)
- PASS:
  - Auth exists in system.
  - README includes role credential matrix with username + password for all listed roles.

## Engineering Quality
Tech stack clarity:
- Strong: frontend/backend/db/container stack clearly listed.

Architecture explanation:
- Moderate: high-level architecture stated; deeper service/module interaction details are brief.

Testing instructions:
- Good baseline: `run_tests.sh` usage documented.
- Could be stronger: expected outputs and minimal acceptance criteria not explicit.

Security/roles:
- Strong: role model and credentials documented.

Workflow explanation:
- Moderate-strong: lifecycle/business capabilities summarized at top.

Presentation quality:
- Strong and readable.

## High Priority Issues
1. Missing explicit verification procedure with deterministic success criteria (strict hard gate impact).

## Medium Priority Issues
1. Architecture section is high-level; lacks concise request flow/module boundary diagram or sequence.
2. Testing section does not specify quick smoke checks per major subsystem.

## Low Priority Issues
1. Could add troubleshooting section for common startup/test failures.
2. Could add environment variable reference table for override scenarios.

## Hard Gate Failures
1. Verification method not explicit enough for strict-mode reproducibility.

## README Verdict
- PARTIAL PASS


# Final Verdicts
- Test Coverage Audit Verdict: PASS (strong coverage and sufficiency, with noted depth gaps in isolated unit layers).
- README Audit Verdict: PARTIAL PASS (fails strict verification-method hard gate).

## Overall Combined Verdict
- Combined status: PARTIAL PASS
- Reason: test quality/coverage is strong, but README strict compliance is blocked by missing deterministic verification steps.

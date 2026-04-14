# Static Delivery Acceptance and Architecture Audit

## 1. Verdict
Overall conclusion: Partial Pass

Reason: The repository is substantial and implements many core flows (RBAC, lifecycle transitions, import queue, connector signing, inventory/audit, export approvals), but there are material requirement-fit gaps and several high-severity risks against the prompt, especially around local-time scheduling fidelity in the Yew UI, missing optional message-queue connector interface, and missing resumable import retry semantics.

## 2. Scope and Static Verification Boundary
What was reviewed:
- Backend architecture, routing, auth/RBAC/middleware, service/repository layers, migration schema, config loading.
- Frontend route structure, guards, role-based navigation, key feature pages (resources/lodgings/inventory/import-export/login).
- Unit tests and API integration tests statically (existence, intent, and assertions).
- Documentation and run/test instructions.

What was not reviewed:
- Runtime behavior under real deployment/network/browser conditions.
- Actual Docker bring-up, DB connectivity in this environment, browser rendering behavior, and performance at scale.

What was intentionally not executed:
- Project startup.
- Docker services.
- Tests.
- Any external integrations.

Claims requiring manual verification:
- End-to-end runtime correctness of scheduled publishing timing and timezone conversion.
- Actual reliability and throughput at 10,000-row import jobs.
- Operational TLS/certificate handling in deployed environments.
- Visual consistency and interaction quality in real browser/device combinations.

## 3. Repository / Requirement Mapping Summary
Prompt core goal and constraints mapped:
- Role-based destination operations portal with resource and lodging lifecycle governance.
- Facility-scoped inventory operations with near-expiry and transaction audit.
- Offline import/export with queue/progress/retry/rollback, plus signed local connector ingest.
- Security controls: auth/RBAC, CSRF, encryption, upload allowlists, anti-replay/idempotency, and masking.

Main implementation areas mapped:
- Backend: Actix routes and layered service/repository model (backend/src/api/mod.rs:10, backend/src/lib.rs:1).
- Frontend: Yew pages, route guards, and role-tailored sidebar (frontend/src/components/sidebar.rs:75, frontend/src/components/route_guard.rs:16).
- Tests: separate API and unit crates with broad module coverage declarations (API_tests/src/lib.rs:1, unit_tests/src/lib.rs:1).

## 4. Section-by-section Review

### 4.1 Hard Gates

#### 4.1.1 Documentation and static verifiability
Conclusion: Pass

Rationale:
- Clear startup and test instructions are present.
- Project structure and route/module declarations are statically coherent.
- A reviewer can attempt verification without rewriting core code.

Evidence:
- README quick start and service endpoints: README.md:19
- Test instructions and script reference: README.md:68
- Route registration and endpoint map in code: backend/src/api/mod.rs:10
- Test script exists: run_tests.sh:1

Manual verification note:
- Runtime validity of all documented commands is Manual Verification Required.

#### 4.1.2 Material deviation from Prompt
Conclusion: Partial Pass

Rationale:
- Major prompt-aligned functionality exists.
- But notable deviations remain: no static evidence of optional on-prem message queue connector interface; retries are re-queue/reprocess, not resumable from prior progress.

Evidence:
- Connector surface appears REST-only: backend/src/api/mod.rs:92, README.md:226
- Import retries re-queue whole job: backend/src/jobs/runner.rs:27, backend/src/jobs/runner.rs:84, backend/src/repository/import_jobs.rs:123

Manual verification note:
- If message-queue support exists outside reviewed source, Cannot Confirm Statistically.

### 4.2 Delivery Completeness

#### 4.2.1 Core requirement coverage
Conclusion: Partial Pass

Rationale:
- Many core requirements are implemented (RBAC, resource/lodging lifecycle, deposit cap, near-expiry, signed connector, export approval/watermark, import rollback and max rows).
- Gaps: resumable retries not evidenced; local-time scheduling semantics are not fully represented in frontend request models; optional MQ connector interface absent in reviewed code.

Evidence:
- Resource lifecycle and scheduled publishing logic: backend/src/service/resources.rs:308, backend/src/jobs/runner.rs:201
- Deposit cap enforcement: backend/src/service/validation.rs:114
- Inventory near-expiry logic: backend/src/repository/inventory.rs:64
- Connector anti-replay/idempotency: backend/src/service/connector.rs:35, backend/src/service/connector.rs:53
- Frontend resource payload lacks timezone offset fields: frontend/src/models/mod.rs:88
- Backend expects tz_offset_minutes: backend/src/model/mod.rs:106

#### 4.2.2 End-to-end deliverable vs partial/demo
Conclusion: Pass

Rationale:
- Complete multi-crate full-stack structure exists with backend/frontend/tests/docs/configuration.
- No evidence of single-file demo-only scope.

Evidence:
- Workspace structure and manifests: README.md:1, backend/Cargo.toml:1, frontend/Cargo.toml:1, API_tests/Cargo.toml:1, unit_tests/Cargo.toml:1

### 4.3 Engineering and Architecture Quality

#### 4.3.1 Structure and module decomposition
Conclusion: Pass

Rationale:
- Reasonable layered backend decomposition and frontend component/page separation.
- Route handlers delegate to services/repositories as expected.

Evidence:
- Backend module layering and exports: backend/src/lib.rs:1
- API routing centralized: backend/src/api/mod.rs:10
- Frontend route/page composition: frontend/src/components/app.rs:13

#### 4.3.2 Maintainability/extensibility
Conclusion: Partial Pass

Rationale:
- Generally maintainable structure and typed DTOs.
- Some extensibility risks: hardcoded warehouse/bin IDs in frontend lot-creation flow; feature flags exist but optional watermark behavior is not visibly toggled in export path.

Evidence:
- Hardcoded inventory location IDs in UI: frontend/src/pages/inventory.rs:126
- Export watermark always generated at approval path: backend/src/service/import_export.rs:71

### 4.4 Engineering Details and Professionalism

#### 4.4.1 Error handling, logging, validation, API design
Conclusion: Partial Pass

Rationale:
- Strong baseline: structured ApiError, validation helpers, logging in critical paths.
- Gaps: media upload endpoint buffers entire file before server-side size rejection (DoS risk under large payloads).

Evidence:
- Structured API error and logging: backend/src/errors.rs:20, backend/src/errors.rs:142
- Validation examples: backend/src/service/validation.rs:13
- Media endpoint reads chunks into memory before service-level check: backend/src/api/media.rs:15, backend/src/api/media.rs:34, backend/src/service/media.rs:68

#### 4.4.2 Product-like vs demo-like
Conclusion: Pass

Rationale:
- Product-like breadth: auth, role-aware UX, workflow states, audit/logging, config center endpoints, import/export queue.

Evidence:
- Route breadth: backend/src/api/mod.rs:10
- UI feature pages and guards: frontend/src/components/app.rs:13

### 4.5 Prompt Understanding and Requirement Fit

#### 4.5.1 Business goal and implicit constraint fit
Conclusion: Partial Pass

Rationale:
- Most core operational workflows are represented.
- Important fit issues remain:
  - Local-time scheduling intent is weakened in frontend model/API shape (missing tz offset input field in request DTOs).
  - Optional on-prem MQ connector interface not evidenced.
  - Retry behavior is bounded but not resumable.

Evidence:
- Backend supports tz offset parsing: backend/src/service/resources.rs:354
- Frontend request structs omit tz_offset_minutes: frontend/src/models/mod.rs:88
- Connector route only inbound REST endpoint: backend/src/api/mod.rs:92
- Requeue semantics without resume pointer: backend/src/repository/import_jobs.rs:123

### 4.6 Aesthetics (frontend)

#### 4.6.1 Visual and interaction quality fit
Conclusion: Cannot Confirm Statistically

Rationale:
- Static evidence shows deliberate design tokens, role-based navigation, modals, badges, validation messages, and upload previews.
- True visual correctness and interaction quality require runtime rendering/device checks.

Evidence:
- Design system and tokens: frontend/style.css:1
- Role-tailored sidebar and states: frontend/src/components/sidebar.rs:75
- Media previews and inline errors: frontend/src/pages/resources.rs:719

Manual verification note:
- Browser/device rendering and interaction feedback quality is Manual Verification Required.

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High

1) Severity: High
Title: Frontend scheduling payload omits timezone offset required for local-time fidelity
Conclusion: Fail
Evidence:
- Backend request model includes tz_offset_minutes: backend/src/model/mod.rs:106
- Backend parser applies tz offset when present: backend/src/service/resources.rs:371
- Frontend Create/Update DTOs omit tz_offset_minutes: frontend/src/models/mod.rs:88
Impact:
- Scheduled publish times entered in local datetime controls can be interpreted as UTC or ambiguously shifted, violating prompt’s local-time scheduling expectation.
Minimum actionable fix:
- Add tz_offset_minutes to frontend Create/Update resource DTOs and pass browser local offset on submit; add tests for timezone conversion boundaries.

2) Severity: High
Title: Import retry is bounded but not resumable
Conclusion: Fail
Evidence:
- Re-queued jobs return to queued status only: backend/src/repository/import_jobs.rs:123
- Processing restarts from full workbook read each run: backend/src/jobs/runner.rs:61, backend/src/jobs/runner.rs:84
- Progress reset to 0 before parse each run: backend/src/jobs/runner.rs:78
Impact:
- Does not satisfy prompt requirement for resumable retries; large jobs may repeatedly restart from scratch on transient failure.
Minimum actionable fix:
- Persist chunk/row cursor and staged state; resume from last committed chunk on retry; add tests for mid-job failure + resume continuation.

3) Severity: High
Title: Optional local message-queue connector interface not evidenced
Conclusion: Fail
Evidence:
- Connector route surface only includes POST /api/connector/inbound: backend/src/api/mod.rs:92
- README connector API table only documents REST inbound endpoint: README.md:226
Impact:
- Prompt explicitly allows optional on-prem MQ interface; missing implementation reduces integration parity and requirement fit.
Minimum actionable fix:
- Implement and document a local MQ connector adapter (or explicitly document deferred scope and mark requirement unsupported).

4) Severity: High
Title: Inventory lot creation UI hardcodes warehouse/bin IDs
Conclusion: Fail
Evidence:
- Hardcoded warehouse/bin in frontend create-lot request: frontend/src/pages/inventory.rs:126
- Hardcoded fallback facility/bin/warehouse also used in import row defaults: backend/src/jobs/runner.rs:135
Impact:
- Undermines real operational management of warehouse/bin locations, increases data integrity risk, and weakens prompt fit for facility inventory operations.
Minimum actionable fix:
- Add APIs/UI to fetch selectable facility warehouses/bins; remove hardcoded IDs from both UI and import fallback behavior.

### Medium

5) Severity: Medium
Title: Media upload memory exposure before size rejection
Conclusion: Partial Fail
Evidence:
- Multipart handler accumulates full file_data in memory: backend/src/api/media.rs:15, backend/src/api/media.rs:34
- Size limit enforced after buffer build in service: backend/src/service/media.rs:68
Impact:
- Large uploads can consume memory before rejection, increasing DoS risk.
Minimum actionable fix:
- Enforce streaming byte limit in multipart read loop and abort once threshold exceeded.

6) Severity: Medium
Title: Export request endpoint lacks explicit role gate despite sensitive export domain
Conclusion: Partial Fail
Evidence:
- request_export has no require_role check: backend/src/api/import_export.rs:113
- Approval and pending list are role-gated: backend/src/api/import_export.rs:129, backend/src/api/import_export.rs:249
Impact:
- Any authenticated role may request sensitive exports, increasing approval queue noise and potential misuse.
Minimum actionable fix:
- Add explicit request role policy matching business intent, or document why all authenticated roles are permitted.

7) Severity: Medium
Title: Optional watermark behavior appears always-on in approval path
Conclusion: Partial Fail
Evidence:
- Watermark generated unconditionally in service approve_export: backend/src/service/import_export.rs:71
- Feature flag export_watermark exists: backend/config.toml:51
Impact:
- Prompt describes optional watermark; implementation appears mandatory regardless feature switch.
Minimum actionable fix:
- Gate watermark generation with config.features.export_watermark and test both enabled/disabled flows.

### Low

8) Severity: Low
Title: MFA handshake contract between backend and frontend is inconsistent
Conclusion: Partial Fail
Evidence:
- Backend login success response sets mfa_required None: backend/src/api/auth.rs:42
- Frontend checks mfa_required == Some(true), else fallback error-string detection: frontend/src/pages/login.rs:113, frontend/src/pages/login.rs:137
Impact:
- Fragile UX behavior if error text changes; not a direct auth bypass.
Minimum actionable fix:
- Return explicit structured MFA challenge payload or status contract frontend can parse deterministically.

## 6. Security Review Summary

- authentication entry points: Pass
  - Evidence: login/logout/me and session validation paths are explicit (backend/src/api/auth.rs:11, backend/src/service/auth.rs:14, backend/src/middleware/auth_guard.rs:89).

- route-level authorization: Partial Pass
  - Evidence: role checks are broadly present via require_role and RbacContext extraction (backend/src/api/resources.rs:17, backend/src/api/lodgings.rs:36, backend/src/api/inventory.rs:29).
  - Gap: export request endpoint has no explicit role gate (backend/src/api/import_export.rs:113).

- object-level authorization: Partial Pass
  - Evidence: facility-scope checks on resource/lodging/inventory object reads/updates (backend/src/api/resources.rs:38, backend/src/api/lodgings.rs:16, backend/src/api/inventory.rs:15).
  - Gap: some global objects with null facility are denied to scoped users by policy; business intent for null-facility entities requires manual policy confirmation.

- function-level authorization: Partial Pass
  - Evidence: sensitive config endpoints admin-only (backend/src/api/config.rs:14).
  - Gap: export request remains unconstrained by role at handler level (backend/src/api/import_export.rs:113).

- tenant / user isolation: Partial Pass
  - Evidence: scope_facility logic and filtered list behaviors for scoped roles (backend/src/middleware/auth_guard.rs:26, backend/src/api/inventory.rs:62, backend/src/api/resources.rs:64).
  - Gap: UI hardcoded location IDs can undermine correct tenant-scoped operational data entry (frontend/src/pages/inventory.rs:126).

- admin / internal / debug protection: Pass
  - Evidence: metrics requires authenticated context extractor (backend/src/api/metrics.rs:19); config endpoints require admin (backend/src/api/config.rs:14).

## 7. Tests and Logging Review

- Unit tests: Pass (with scope gaps)
  - Evidence of coverage for crypto, validation, state machine, deposit cap, masking, near-expiry (unit_tests/src/lib.rs:1, unit_tests/src/test_deposit_cap.rs:4, unit_tests/src/test_state_machine.rs:10).

- API / integration tests: Pass (with risk gaps)
  - Evidence of auth, RBAC, CSRF, lifecycle, media validation, connector anti-replay, metrics, inventory, import/export paths (API_tests/src/lib.rs:1, API_tests/src/test_security.rs:7, API_tests/src/test_connector.rs:17).

- Logging categories / observability: Partial Pass
  - Evidence: tracing in API errors, job runner, and connector/import paths (backend/src/errors.rs:142, backend/src/jobs/runner.rs:30, backend/src/service/connector.rs:64).
  - Gap: no static evidence of structured alert routing beyond logs.

- Sensitive-data leakage risk in logs / responses: Partial Pass
  - Positive: error responses are structured and do not dump full internals by default (backend/src/errors.rs:20).
  - Risk: export payload masking logic covers selected plain-text keys only; ciphertext/other sensitive fields may still be exported depending on table columns (backend/src/api/import_export.rs:256).

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist: yes (unit_tests/src/lib.rs:1).
- API/integration tests exist: yes (API_tests/src/lib.rs:1).
- Frameworks: Rust built-in test harness with tokio async tests.
- Test entry points: cargo test -p unit_tests, cargo test -p api_tests (run_tests.sh:87, run_tests.sh:95).
- Documentation provides test commands: yes (README.md:68, run_tests.sh:1).

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Auth happy path and 401 failures | API_tests/src/test_auth.rs:6, API_tests/src/test_auth.rs:62 | 200 on success, 401 on unauthenticated me | sufficient | none major | add lockout/rate-limit tests if required |
| CSRF enforcement | API_tests/src/test_security.rs:7 | POST without token -> 403; valid token -> success | sufficient | none major | add CSRF token expiry test |
| Role authorization (403) | API_tests/src/test_rbac.rs:4 | Clinician blocked from lodging create, clerk blocked from resources | basically covered | not exhaustive for every endpoint | add table-driven endpoint-role matrix tests |
| Facility/object isolation | API_tests/src/test_security.rs:99 | Clinician denied cross-facility lot access | basically covered | limited object permutations | add resource/lodging cross-facility object tests |
| Resource lifecycle transitions | API_tests/src/test_resources.rs:116, unit_tests/src/test_state_machine.rs:10 | transition success/denial by role and state | sufficient | scheduler-time edge not full | add timezone-sensitive schedule transition tests |
| Lodging deposit cap and rent-change approval | API_tests/src/test_lodgings.rs:27, API_tests/src/test_lodgings.rs:175, unit_tests/src/test_deposit_cap.rs:4 | 1.5x accepted, >1.5x rejected, approval flow updates rent | sufficient | none major | add concurrent approve/reject conflict test |
| Inventory reserve/conflict and near-expiry | API_tests/src/test_inventory.rs:86, API_tests/src/test_inventory.rs:156 | over-reserve 409, near_expiry filter behavior | basically covered | limited date boundary cases | add exact 30-day boundary and timezone date rollover tests |
| Media allowlist and MIME sniff | API_tests/src/test_media.rs:64 | mismatched JPG/PDF -> MIME_MISMATCH | basically covered | no 50MB boundary test | add file-size limit test at 50MB and 50MB+1 |
| Connector anti-replay/idempotency | API_tests/src/test_connector.rs:17, API_tests/src/test_connector.rs:61 | valid signed request 200, replay nonce 409, bad signature 401 | sufficient | no timestamp boundary fuzzing | add +/-300s edge tests and clock skew matrix |
| Import type validation and export approval flow | API_tests/src/test_import_export.rs:4, API_tests/src/test_import_export.rs:45 | pre-approval export blocked; non-xlsx rejected | insufficient | no resumable retry behavior test, no 10k+ rows API test | add integration tests for retry resume semantics and 10,001-row rejection path |
| Metrics and health endpoints | API_tests/src/test_metrics_health.rs:21 | Prometheus text and metric labels asserted | basically covered | no authorization-negative test for metrics | add unauthenticated metrics 401 test |

### 8.3 Security Coverage Audit
- authentication: sufficient coverage
  - Covered by login success/failure and me unauthenticated checks (API_tests/src/test_auth.rs:6, API_tests/src/test_auth.rs:62).

- route authorization: basically covered
  - Covered for multiple representative endpoints and roles (API_tests/src/test_rbac.rs:4).
  - Severe defects could remain on untested endpoint-role pairs.

- object-level authorization: basically covered
  - Covered for cross-facility lot access denial (API_tests/src/test_security.rs:99).
  - Resource/lodging object-level permutations remain partially untested.

- tenant / data isolation: basically covered
  - Clinician list/get scope tests exist (API_tests/src/test_security.rs:163).
  - Hardcoded frontend lot location IDs remain untested as an isolation quality risk.

- admin / internal protection: basically covered
  - Config admin-only tested (API_tests/src/test_security.rs:484), metrics positive path tested (API_tests/src/test_metrics_health.rs:21).
  - Missing explicit negative metrics authorization test.

### 8.4 Final Coverage Judgment
Partial Pass

Boundary explanation:
- Major security and workflow paths are tested (auth, CSRF, RBAC examples, state machine, connector replay/signature, inventory conflicts).
- However, uncovered/high-risk areas remain (resumable retry semantics, timezone scheduling correctness, endpoint-role matrix completeness, and media size boundary), meaning severe defects in those areas could still exist while tests pass.

## 9. Final Notes
- This report is static-only and evidence-based; runtime claims are intentionally bounded.
- The strongest acceptance risks are requirement-fit gaps rather than missing project structure.
- Addressing the four high-severity findings should materially improve acceptance confidence.

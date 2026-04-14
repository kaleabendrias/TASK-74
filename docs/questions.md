# Questions and Clarifications Log

This document records requirement ambiguities, implementation interpretations, and final decisions used during delivery. It is intentionally written in a structured review format.

## 1. Offline mode vs third-party login endpoint
Question: The prompt requires a fully offline, on-prem system and also asks for an extensible integration posture. Should external-login-style surfaces be removed entirely, or kept as local extension points?

My Understanding: External cloud identity dependencies must not be required at runtime. Local integration extension points may still exist if they are on-prem only and do not violate offline policy.

Solution: Kept local-only connector and local auth model, with no dependency on external identity providers. Integration is represented by local-network signed ingress and optional on-prem MQ, not cloud login delegation.

---

## 2. Request-signing boundary for authenticated routes
Question: Should every authenticated endpoint require HMAC signatures, or should signing apply specifically to integration boundaries while session-based app flows rely on session+CSRF?

My Understanding: Portal UI/API traffic should use session authentication plus CSRF controls. HMAC signing should harden machine-to-machine integration ingress where replay and duplicate ingestion risk is highest.

Solution: Applied signature, nonce, timestamp, anti-replay window, and idempotency controls to connector inbound flow, while user-driven API routes remain protected by RBAC session context and CSRF.

---

## 3. Password recovery behavior in an offline deployment
Question: Should a full email/SMS password recovery flow be implemented despite offline constraints and no external service dependency requirement?

My Understanding: The prompt does not mandate an external recovery channel and prioritizes offline operation. A local-admin-governed credential lifecycle is acceptable for this release scope.

Solution: Implemented local username/password with MFA controls and session security. External email/SMS recovery dependency was not introduced.

---

## 4. Health endpoint data exposure model
Question: Should health checks expose detailed service metadata publicly, or should details be restricted to authenticated operational roles?

My Understanding: Public liveness should be minimal and safe for probes. Detailed operational metadata should be protected.

Solution: Split health into a public minimal liveness endpoint and an authenticated readiness endpoint carrying detailed metadata.

---

## 5. Export approval semantics and second-person rule
Question: Prompt requires second-person approval for sensitive exports. Does role-only approval suffice, or must requester self-approval be explicitly blocked?

My Understanding: True second-person control requires explicit requester != approver enforcement.

Solution: Added explicit self-approval rejection in export approval logic and retained role restrictions for review authority.

---

## 6. Import processing shape: all-or-nothing vs partial commit
Question: Should import persist valid rows when some rows fail, or require strict pre-validation and staged final commit?

My Understanding: Prompt requires rollback-safe, deterministic import behavior with staging and single final commit.

Solution: Implemented XLSX validation and staging workflow with resumable progress cursor, retries, and final transactional commit into target tables.

---

## 7. Scheduled publication time interpretation
Question: Prompt allows local-time scheduling examples. Should backend assume UTC-only input or support local-time capture from UI?

My Understanding: UI can capture local datetime; backend should accept local representation with offset metadata to avoid ambiguous publication timing.

Solution: Frontend submits timezone offset with schedule input and backend normalizes scheduled publish time to UTC for scheduler evaluation.

---

## 8. Facility data scope and global roles
Question: When facility_id is missing on content entities, should facility-scoped users be allowed to view such records or blocked by default?

My Understanding: To avoid unintended leakage, constrained roles should not auto-inherit access to unscoped entities unless explicitly defined as global.

Solution: Facility-scoped checks deny access to mismatched or unassigned facility records, while unrestricted roles can operate globally.

---

## 9. Media validation strictness
Question: Is client-side extension validation sufficient, or should server-side sniffing and integrity checks be mandatory?

My Understanding: Client-side checks improve UX but are not trust boundaries; server-side allowlist, MIME sniffing, and checksum persistence are required.

Solution: Added backend file-type sniff validation, extension-to-MIME consistency checks, max-size limits, and SHA-256 checksum storage.

---

## 10. Real-time import progress delivery model
Question: Should progress be polling-only, or provide push semantics for operational visibility?

My Understanding: Prompt asks for progress visibility and retry handling; SSE push with polling fallback provides robust local-network behavior.

Solution: Added import status SSE stream endpoint and frontend EventSource handling with graceful polling fallback.

---

## 11. Operational configuration center scope
Question: Should config center be static constants or runtime profile-aware parameters with controlled updates?

My Understanding: Prompt expects configuration center behavior across environments and feature switches.

Solution: Implemented profile-scoped config parameter persistence endpoints with Administrator-only access controls.

---

## 12. Local-network connector transport options
Question: Prompt references local-network connectors and optional queue interface. Should only REST be delivered, or should queue-style ingestion also be represented?

My Understanding: Both direct REST ingestion and optional local message-queue ingestion should be available without external SaaS dependency.

Solution: Implemented signed REST inbound connector and optional local MQ connector path with TCP/AMQP transport support under local deployment settings.

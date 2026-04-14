# Fix Check Report for audit_report-1.md

Static-only reassessment date: 2026-04-14
Reference report: [.tmp/audit_report-1.md](.tmp/audit_report-1.md)

## Summary
- Total issues checked: 8
- Fixed: 8
- Partially fixed: 0
- Not fixed: 0

## Issue-by-Issue Status

| # | Original Issue | Previous Severity | Current Status | Evidence | Notes |
|---|---|---|---|---|---|
| 1 | Frontend scheduling payload omitted timezone offset | High | Fixed | [repo/frontend/src/models/mod.rs](repo/frontend/src/models/mod.rs#L88), [repo/frontend/src/models/mod.rs](repo/frontend/src/models/mod.rs#L100), [repo/frontend/src/models/mod.rs](repo/frontend/src/models/mod.rs#L128), [repo/frontend/src/pages/resources.rs](repo/frontend/src/pages/resources.rs#L447), [repo/frontend/src/pages/resources.rs](repo/frontend/src/pages/resources.rs#L467), [repo/backend/src/model/mod.rs](repo/backend/src/model/mod.rs#L106) | Frontend request DTOs now include tz_offset_minutes and submit logic sends browser offset. Backend already accepts tz_offset_minutes. |
| 2 | Import retry was bounded but not resumable | High | Fixed | [repo/backend/src/jobs/runner.rs](repo/backend/src/jobs/runner.rs#L56), [repo/backend/src/jobs/runner.rs](repo/backend/src/jobs/runner.rs#L74), [repo/backend/src/jobs/runner.rs](repo/backend/src/jobs/runner.rs#L105), [repo/backend/src/jobs/runner.rs](repo/backend/src/jobs/runner.rs#L194), [repo/backend/src/jobs/runner.rs](repo/backend/src/jobs/runner.rs#L210), [repo/backend/src/jobs/runner.rs](repo/backend/src/jobs/runner.rs#L237), [repo/backend/src/repository/import_jobs.rs](repo/backend/src/repository/import_jobs.rs#L122), [repo/backend/src/repository/import_jobs.rs](repo/backend/src/repository/import_jobs.rs#L125) | Resume cursor and staging table persistence were added; retries can continue from committed chunk cursor. |
| 3 | Optional local message-queue connector interface not evidenced | High | Fixed | [repo/backend/src/config/mod.rs](repo/backend/src/config/mod.rs#L154), [repo/backend/src/main.rs](repo/backend/src/main.rs#L77), [repo/backend/src/main.rs](repo/backend/src/main.rs#L84), [repo/backend/src/service/mq_connector.rs](repo/backend/src/service/mq_connector.rs#L1), [repo/backend/src/service/mq_connector.rs](repo/backend/src/service/mq_connector.rs#L120), [repo/README.md](repo/README.md#L230) | Optional MQ connector now exists with TCP and AMQP transports and startup wiring. |
| 4 | Inventory lot creation used hardcoded warehouse/bin IDs | High | Fixed | [repo/frontend/src/pages/inventory.rs](repo/frontend/src/pages/inventory.rs#L34), [repo/frontend/src/pages/inventory.rs](repo/frontend/src/pages/inventory.rs#L72), [repo/frontend/src/pages/inventory.rs](repo/frontend/src/pages/inventory.rs#L91), [repo/frontend/src/pages/inventory.rs](repo/frontend/src/pages/inventory.rs#L167), [repo/frontend/src/pages/inventory.rs](repo/frontend/src/pages/inventory.rs#L179), [repo/backend/src/api/mod.rs](repo/backend/src/api/mod.rs#L53), [repo/backend/src/repository/inventory.rs](repo/backend/src/repository/inventory.rs#L18), [repo/backend/src/jobs/runner.rs](repo/backend/src/jobs/runner.rs#L159) | UI now loads/selects warehouses and bins from APIs; import validation now requires facility_id/warehouse_id/bin_id and no longer silently defaults missing IDs. |
| 5 | Media upload buffered full payload before size rejection | Medium | Fixed | [repo/backend/src/api/media.rs](repo/backend/src/api/media.rs#L34), [repo/backend/src/api/media.rs](repo/backend/src/api/media.rs#L42), [repo/backend/src/errors.rs](repo/backend/src/errors.rs#L97) | Upload stream now aborts as soon as size exceeds configured max and returns 413 payload-too-large error. |
| 6 | Export request endpoint lacked explicit role gate | Medium | Fixed | [repo/backend/src/api/import_export.rs](repo/backend/src/api/import_export.rs#L115), [repo/backend/src/api/import_export.rs](repo/backend/src/api/import_export.rs#L120) | request_export now requires Administrator or Reviewer role. |
| 7 | Watermark behavior appeared always-on | Medium | Fixed | [repo/backend/src/api/import_export.rs](repo/backend/src/api/import_export.rs#L127), [repo/backend/src/api/import_export.rs](repo/backend/src/api/import_export.rs#L142), [repo/backend/src/service/import_export.rs](repo/backend/src/service/import_export.rs#L52), [repo/backend/src/service/import_export.rs](repo/backend/src/service/import_export.rs#L73) | Watermark generation is now feature-flag gated via export_watermark. |
| 8 | MFA backend/frontend handshake contract was inconsistent | Low | Fixed | [repo/backend/src/api/auth.rs](repo/backend/src/api/auth.rs#L13), [repo/backend/src/api/auth.rs](repo/backend/src/api/auth.rs#L30), [repo/backend/src/api/auth.rs](repo/backend/src/api/auth.rs#L36), [repo/frontend/src/pages/login.rs](repo/frontend/src/pages/login.rs#L113), [repo/frontend/src/pages/login.rs](repo/frontend/src/pages/login.rs#L135) | Backend now returns deterministic mfa_required challenge payload; frontend uses structured field and no longer relies on fragile error-string MFA detection. |

## Final Check Result
All 8 issues listed in [.tmp/audit_report-1.md](.tmp/audit_report-1.md) are fixed based on static source evidence.

## Boundary Note
This is a static fix check only. Runtime correctness, migration compatibility in deployed environments, and behavior under load still require manual verification.
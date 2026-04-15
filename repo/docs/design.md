# Design Decisions

Implementation and architecture choices made during development.
Unresolved requirements belong in `questions.md`.

---

## Architecture

### Backend: Actix-web + Diesel + PostgreSQL
Chosen for compile-time query safety (Diesel), mature async HTTP (Actix-web), and
strong Rust ecosystem support. PostgreSQL selected for its JSON support, row-level
security capability, and reliable UUID generation via `gen_random_uuid()`.

### Frontend: Yew (WASM)
Yew allows the shared domain logic crate (`frontend_logic`) to be compiled for both
the WASM frontend and the native test runner. This eliminates duplication of
validation and routing logic between frontend and test suites.

### Shared `frontend_logic` crate
Pure-Rust library compiled without WASM features. Both the Yew frontend and the
`frontend_tests` native test binary depend on it. This means any regression in
validation, routing, or state-machine logic is caught by `cargo test` without
a browser.

---

## Security

### TLS: self-signed certificate in dev/test
A self-signed certificate is generated at container build time (2048-bit RSA, 10-year
validity). All test clients use `danger_accept_invalid_certs(true)` to connect.
One test (`strict_tls_client_rejects_self_signed_cert`) verifies that a standard TLS
client *does* reject this certificate, proving TLS is active and the bypass is
load-bearing.
Production deployments must replace the cert with one signed by a trusted CA.

### Session tokens via HttpOnly Secure cookie + Bearer header
The session token is set as `HttpOnly; Secure; SameSite=Strict`. Test clients cannot
read it via document.cookie; they extract it from the `Set-Cookie` response header
and inject it as a `Bearer` token. This is the same approach used by the E2E seed
scripts.

### CSRF protection
All mutating endpoints (`POST`, `PUT`, `DELETE`) require an `X-CSRF-Token` header
matching the per-session CSRF token returned at login. `GET` requests are exempt.
The `login` and `health` endpoints are exempt from CSRF to avoid bootstrapping
problems.

### Argon2id parameters (dev/test)
Test environments use reduced parameters (memory=4 MiB, iterations=1) to keep test
runtime acceptable. Production parameters are set via environment variables and
default to 64 MiB / 3 iterations / parallelism 4.

---

## Database

### Single database for backend + tests
The test-runner uses the same `tourism_portal` database as the backend. API tests
call `seed_users()` which clears all tables and recreates standard test accounts
before each test file. This ensures API tests authenticate against the same database
the backend reads from — a separate test database would require the backend to
dynamically switch connection strings, which adds complexity with no safety benefit
given the pre-run `docker compose down -v` that guarantees a clean slate.

### Migrations on startup
`run_migrations()` is called by the backend on startup (before accepting requests)
and by `setup_pool()` in API tests. Unit tests that connect directly use
`tourism_portal` which is already migrated by the backend before tests run.

---

## Testing

### Coverage scope: `frontend_logic` + `frontend_tests` only
Tarpaulin instruments the three pure-Rust packages. Backend API handler code
(`backend/src/**`) is excluded: ptrace-based instrumentation cannot follow async
Actix-web handlers running under live TLS without corrupting the TLS state machine.
API test pass-rate (152 tests across all routes) serves as the request-path coverage
proxy. The combined coverage gate is ≥90%.

### Test execution order
Unit tests → Frontend tests → Backend health check → API integration tests.
Unit tests run first because they are the fastest and catch pure-logic regressions
without needing the network. API tests run last because they depend on the backend
being healthy and on `seed_users()` having run.

### Backend health as hard gate
The host-side backend health check (before entering the test-runner container) is a
hard `exit 1` on timeout. The in-container health check before API tests is also a
hard `exit 1`. A backend that never becomes healthy would previously allow tests to
start and produce misleading connection-error failures; the hard gate surfaces the
real problem immediately.

### E2E credentials
E2E tests use the standard seeded accounts (`admin/Admin@2024`, `publisher/Pub@2024`,
etc.). These are created by the backend's `seed_defaults()` on first startup (when
the `users` table is empty). Because `seed_users()` in API tests creates users with
`password=testpassword` (different password), E2E must run in a fresh container
where `seed_defaults()` has run but API tests have not yet overwritten the users.
The `--no-e2e` flag allows CI runs that only need API-level correctness.

---

## Export

### PII masking
Exports are generated as `.xlsx` files. Address and personal-data fields are masked
before the workbook is serialised. The watermark is written to a dedicated `Metadata`
sheet so it survives normal editing of data sheets. Export bytes begin with
`PK\x03\x04` (ZIP magic); tests assert this to catch serialisation corruption.

### Approval gate
An export requested by user A must be approved by a different user before the
download link becomes available. Self-approval is blocked at the service layer.
Only the original requester and Administrators may download an approved export.

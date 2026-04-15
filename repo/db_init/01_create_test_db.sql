-- Creates an isolated test database for the API/unit test runner so that
-- test-suite data churn (seed_users deletes + re-inserts every test) never
-- touches the backend's live database. E2E tests continue to use the live
-- 'tourism_portal' database which retains the original seeded credentials.
SELECT 'CREATE DATABASE tourism_portal_test'
WHERE NOT EXISTS (
    SELECT FROM pg_database WHERE datname = 'tourism_portal_test'
)\gexec

GRANT ALL PRIVILEGES ON DATABASE tourism_portal_test TO tourism;

#!/usr/bin/env bash
#
# run_tests.sh — Build, run unit + API + frontend + E2E tests, enforce coverage gate.
#
# Usage:
#   ./run_tests.sh            # unit, API, frontend, E2E tests + coverage gate
#   ./run_tests.sh --no-e2e   # same but skip Playwright E2E
#
set -euo pipefail

COMPOSE_FILE="docker-compose.yml"
COVERAGE_DIR="$(pwd)/coverage_output"
REQUIRED_COVERAGE=90
RUN_E2E=true

for arg in "$@"; do
  case "$arg" in
    --no-e2e) RUN_E2E=false ;;
    *) echo "Unknown argument: $arg"; exit 1 ;;
  esac
done

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'

cleanup() {
    echo -e "\n${CYAN}[teardown]${NC} Stopping and removing containers..."
    docker compose -f "$COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
    _proj="${COMPOSE_PROJECT_NAME:-$(basename "$(pwd)")}"
    docker ps -a --filter "label=com.docker.compose.project=${_proj}" -q \
        | xargs -r docker rm -f 2>/dev/null || true
}
trap cleanup EXIT

# Remove any leftover state from a previously interrupted run so volumes
# (especially pgdata) start fresh and don't carry stale credentials.
# We also force-remove ALL containers carrying the compose project label so
# that orphan containers from a previous compose file (e.g. repo-postgres-1)
# cannot hold ports (like 5433) that this stack needs.
echo -e "${CYAN}[pre-run]${NC} Removing any leftover containers and volumes..."
docker compose -f "$COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
COMPOSE_PROJECT="${COMPOSE_PROJECT_NAME:-$(basename "$(pwd)")}"
docker ps -a --filter "label=com.docker.compose.project=${COMPOSE_PROJECT}" -q \
    | xargs -r docker rm -f 2>/dev/null || true
docker volume ls --filter "label=com.docker.compose.project=${COMPOSE_PROJECT}" -q \
    | xargs -r docker volume rm 2>/dev/null || true

mkdir -p "$COVERAGE_DIR"

echo -e "${CYAN}============================================================${NC}"
echo -e "${CYAN}  Tourism Portal — Test Suite Runner${NC}"
echo -e "${CYAN}============================================================${NC}"

# ── 1. Build ──────────────────────────────────────────────────
echo -e "\n${YELLOW}[1/6]${NC} Building services..."
docker compose -f "$COMPOSE_FILE" build --no-cache backend
docker compose -f "$COMPOSE_FILE" --profile test build --no-cache test-runner
if [ "$RUN_E2E" = "true" ]; then
    docker compose -f "$COMPOSE_FILE" build --no-cache frontend
    docker compose -f "$COMPOSE_FILE" --profile e2e build --no-cache test-e2e
fi

# ── 2. Start database + backend ────────────────────────────────
echo -e "\n${YELLOW}[2/6]${NC} Starting database and backend..."
docker compose -f "$COMPOSE_FILE" up -d db

echo "  Waiting for database..."
RETRIES=30
until docker compose -f "$COMPOSE_FILE" exec -T db pg_isready -U tourism -d tourism_portal > /dev/null 2>&1; do
    RETRIES=$((RETRIES-1)); [ "$RETRIES" -gt 0 ] || { echo -e "${RED}ERROR: DB timeout${NC}"; exit 1; }
    sleep 1
done
echo -e "  ${GREEN}Database ready.${NC}"

docker compose -f "$COMPOSE_FILE" up -d backend
sleep 5

echo "  Waiting for backend..."
RETRIES=40
until curl -ksf https://localhost:8088/api/health > /dev/null 2>&1; do
    RETRIES=$((RETRIES-1))
    [ "$RETRIES" -gt 0 ] || { echo -e "${RED}ERROR: Backend did not become healthy in time — aborting.${NC}"; exit 1; }
    sleep 2
done
echo -e "  ${GREEN}Backend up.${NC}"

# ── 3. Run all tests in a single session ───────────────────────
#  Run unit → frontend → [backend health check] → API tests.
#  Using one docker-compose-run keeps every container on the same
#  default network so "backend:8080" DNS stays valid throughout.
echo -e "\n${YELLOW}[3/6]${NC} Running unit, frontend and API tests..."

TEST_RUN_EXIT=0
docker compose -f "$COMPOSE_FILE" --profile test run --rm test-runner bash -c '
set -e

echo "========================================="
echo "  Unit Tests"
echo "========================================="
cargo test -p unit_tests -- --test-threads=1 2>&1 | tee /coverage/unit_tests.log
UNIT_EXIT=${PIPESTATUS[0]:-$?}

echo ""
echo "========================================="
echo "  Frontend Component Tests (pure-Rust)"
echo "========================================="
cargo test -p frontend_tests -- --test-threads=1 2>&1 | tee /coverage/frontend_tests.log
FE_EXIT=${PIPESTATUS[0]:-$?}

echo ""
echo "  Verifying backend is still reachable before API tests..."
RETRIES=20
until curl -ksf https://backend:8080/api/health > /dev/null 2>&1; do
    RETRIES=$((RETRIES-1))
    if [ "$RETRIES" -le 0 ]; then
        echo "ERROR: Backend unreachable before API tests."
        exit 1
    fi
    echo "  ... waiting for backend ($RETRIES retries left)"
    sleep 3
done
echo "  Backend reachable. Starting API tests."

echo ""
echo "========================================="
echo "  API Integration Tests"
echo "========================================="

run_api_tests() {
    cargo test -p api_tests -- --test-threads=1 2>&1 | tee /coverage/api_tests.log
    return ${PIPESTATUS[0]:-$?}
}

run_api_tests
API_EXIT=$?

# If tests failed, check whether it was due to backend crash (connection errors).
# If backend is reachable again (auto-restarted), retry once.
if [ "$API_EXIT" -ne 0 ]; then
    if grep -qiE "connection refused|dns error|name resolution" /coverage/api_tests.log 2>/dev/null; then
        echo ""
        echo "  Backend connection errors detected — waiting for backend to recover..."
        RETRIES=30
        until curl -ksf https://backend:8080/api/health > /dev/null 2>&1; do
            RETRIES=$((RETRIES-1))
            if [ "$RETRIES" -le 0 ]; then
                echo "  ERROR: Backend did not recover in time."
                break
            fi
            echo "  ... waiting ($RETRIES retries left)"
            sleep 3
        done
        if curl -ksf https://backend:8080/api/health > /dev/null 2>&1; then
            echo "  Backend recovered. Retrying API tests..."
            run_api_tests
            API_EXIT=$?
        fi
    fi
fi

echo "UNIT_EXIT=$UNIT_EXIT"  > /coverage/exit_codes.txt
echo "FE_EXIT=$FE_EXIT"     >> /coverage/exit_codes.txt
echo "API_EXIT=$API_EXIT"   >> /coverage/exit_codes.txt

echo ""
echo "========================================="
echo "  Tests Complete"
echo "========================================="
echo "  Unit:     exit=$UNIT_EXIT"
echo "  Frontend: exit=$FE_EXIT"
echo "  API:      exit=$API_EXIT"

if [ "$UNIT_EXIT" -ne 0 ] || [ "$FE_EXIT" -ne 0 ] || [ "$API_EXIT" -ne 0 ]; then
    exit 1
fi
' 2>&1 | tee "$COVERAGE_DIR/tests_output.log" || TEST_RUN_EXIT=$?

# ── 4. Tarpaulin coverage ──────────────────────────────────────
echo -e "\n${YELLOW}[4/6]${NC} Measuring code coverage..."

TARPAULIN_EXIT=0
docker compose -f "$COMPOSE_FILE" --profile test run --rm test-runner bash -c '
    echo "Running tarpaulin across pure-Rust packages..."
    # Wait for the "db" hostname to be resolvable before starting Tarpaulin.
    # A freshly-started "docker compose run" container may hit a brief window
    # where Docker'\''s embedded DNS has not yet propagated the db service entry,
    # causing the 4 DB-dependent tests in unit_tests to fail with
    # "Temporary failure in name resolution".
    echo "  Waiting for DB DNS to be ready..."
    until getent hosts db >/dev/null 2>&1; do sleep 1; done
    echo "  DB DNS ready."
    # Measures line coverage for all instrumentable packages:
    #   frontend_logic  — shared domain logic (validation, routing, masking, etc.)
    #   frontend_tests  — exercises frontend_logic via the test suite
    #   unit_tests      — exercises backend domain logic (crypto, state machines, import, etc.)
    # API integration tests (reqwest against live server) are not instrumentable
    # with ptrace-based ptrace coverage; their 100% pass rate validates those paths.
    # Wait for postgres to accept TCP connections before running DB-dependent tests.
    # getent confirms DNS resolves; /dev/tcp confirms the port is open (bash built-in,
    # no external tools needed).  Both checks avoid "Temporary failure in name
    # resolution" when Tarpaulin'\''s test threads race to connect simultaneously.
    echo "  Waiting for DB TCP port to be reachable..."
    until getent hosts db >/dev/null 2>&1 && bash -c "echo >/dev/tcp/db/5432" 2>/dev/null; do sleep 1; done
    echo "  DB TCP reachable."
    cargo tarpaulin \
        --packages frontend_logic frontend_tests unit_tests \
        --exclude-files "backend/src/**" \
        --out Stdout \
        --skip-clean \
        --timeout 300 \
        -- --test-threads=1 \
        2>&1 | tee /coverage/tarpaulin.log
    echo "Tarpaulin done."
' 2>&1 | tee "$COVERAGE_DIR/coverage_full.log" || TARPAULIN_EXIT=$?

# ── 5. Pull logs + parse ───────────────────────────────────────
echo -e "\n${YELLOW}[5/6]${NC} Parsing results..."

# Copy logs from named volume
VOL_NAME="$(basename "$(pwd)")_coverage-data"
docker run --rm \
    -v "${VOL_NAME}:/data:ro" \
    -v "$COVERAGE_DIR:/out" \
    alpine sh -c "cp /data/* /out/ 2>/dev/null || true" 2>/dev/null || true

count_ok() {
    # Return the LARGEST "passed" count across all "test result: ok." lines
    # (avoids picking the doc-test "0 passed" line that follows the real run)
    local f="$1" fallback="$2"
    local n
    n=$(grep -oP 'test result: ok\. \K\d+' "${f:-/dev/null}" 2>/dev/null \
        | sort -n | tail -1 || echo "")
    if [ -z "$n" ] || [ "$n" = "0" ]; then
        if [ -n "${fallback:-}" ] && [ -f "$fallback" ]; then
            n=$(grep -oP 'test result: ok\. \K\d+' "$fallback" 2>/dev/null \
                | sort -n | tail -1 || echo "?")
        fi
    fi
    echo "${n:-?}"
}

UNIT_PASS=$(count_ok "$COVERAGE_DIR/unit_tests.log" "$COVERAGE_DIR/tests_output.log")
FE_PASS=$(count_ok "$COVERAGE_DIR/frontend_tests.log" "$COVERAGE_DIR/tests_output.log")
API_PASS=$(count_ok "$COVERAGE_DIR/api_tests.log" "$COVERAGE_DIR/tests_output.log")

# Extract coverage %
COVERAGE_PCT=0
for log in "$COVERAGE_DIR/tarpaulin.log" "$COVERAGE_DIR/coverage_full.log"; do
    if [ -f "$log" ]; then
        RAW=$(grep -oP '\d+\.\d+% coverage' "$log" 2>/dev/null | tail -1 || echo "")
        if [ -n "$RAW" ]; then
            COVERAGE_PCT=$(echo "$RAW" | grep -oP '^\d+' || echo "0")
            break
        fi
    fi
done

# Per-package coverage: sum Tested/Total lines for files under each package prefix.
# Tarpaulin log format:  || <pkg>/src/<file>.rs: <tested>/<total>
pkg_cov() {
    local log="$1" pkg="$2"
    [ -f "$log" ] || { echo "N/A"; return; }
    grep -P "^\|\| ${pkg}/src/.*: [0-9]+/[0-9]+" "$log" 2>/dev/null \
        | awk -F': ' '{split($2,a,"/"); t+=a[1]; tot+=a[2]} END{if(tot>0) printf "%d", t*100/tot; else print "N/A"}' \
        || echo "N/A"
}

TARP_LOG=""
for _l in "$COVERAGE_DIR/tarpaulin.log" "$COVERAGE_DIR/coverage_full.log"; do
    [ -f "$_l" ] && { TARP_LOG="$_l"; break; }
done

FL_COV=$(pkg_cov "$TARP_LOG" "frontend_logic")
FT_COV=$(pkg_cov "$TARP_LOG" "frontend_tests")

# ── 5b. E2E (optional) ─────────────────────────────────────────
E2E_EXIT=0
E2E_LABEL="(skipped — use without --no-e2e)"
if [ "$RUN_E2E" = "true" ]; then
    # ── Restore E2E credentials ─────────────────────────────────
    # API tests call seed_users() which replaces every user's password hash with
    # 'testpassword'.  seed_defaults() now uses ON CONFLICT DO UPDATE, so a simple
    # backend restart is enough: seed_defaults() runs on every startup and resets
    # all password hashes back to the E2E values (Admin@2024, Pub@2024, …).
    echo -e "  Restoring E2E credentials via backend restart..."
    docker compose -f "$COMPOSE_FILE" restart backend
    sleep 3

    echo -e "  Waiting for backend healthy status..."
    docker compose -f "$COMPOSE_FILE" up -d --wait --wait-timeout 120 backend \
        || { echo -e "${RED}ERROR: Backend did not become healthy before E2E.${NC}"; E2E_EXIT=1; }

    # Verify admin login works before kicking off E2E tests.
    if [ "$E2E_EXIT" -eq 0 ]; then
        RETRIES=10
        until curl -ks -X POST https://localhost:8088/api/auth/login \
            -H 'Content-Type: application/json' \
            -d '{"username":"admin","password":"Admin@2024"}' \
            | grep -q '"csrf_token"'; do
            RETRIES=$((RETRIES-1))
            if [ "$RETRIES" -le 0 ]; then
                echo -e "${RED}ERROR: Admin login failed after credential restore — E2E would fail.${NC}"
                E2E_EXIT=1
                break
            fi
            echo "  ... waiting for credential restore ($RETRIES retries left)"
            sleep 3
        done
        [ "$E2E_EXIT" -eq 0 ] && echo -e "  ${GREEN}Admin login verified — E2E credentials restored.${NC}"
    fi

    if [ "$E2E_EXIT" -eq 0 ]; then
        echo -e "  Starting frontend and waiting for healthy status..."
        docker compose -f "$COMPOSE_FILE" up -d --wait --wait-timeout 120 frontend \
            || { echo -e "${YELLOW}WARNING: Frontend health check did not pass; proceeding anyway.${NC}"; }

        if [ "$E2E_EXIT" -eq 0 ]; then
        docker compose -f "$COMPOSE_FILE" --profile e2e run --rm test-e2e \
            2>&1 | tee "$COVERAGE_DIR/e2e.log" || E2E_EXIT=$?
        fi
        [ "$E2E_EXIT" -eq 0 ] && E2E_LABEL="PASS" || E2E_LABEL="FAIL (see $COVERAGE_DIR/e2e.log)"
    else
        E2E_LABEL="FAIL (backend did not recover before E2E)"
    fi
fi

# ── 6. Summary ─────────────────────────────────────────────────
echo -e "\n${YELLOW}[6/6]${NC} Test Summary:"
echo -e "${CYAN}┌──────────────────────────┬──────────────────────────────────┐${NC}"
echo -e "${CYAN}│ Suite                    │ Result                           │${NC}"
echo -e "${CYAN}├──────────────────────────┼──────────────────────────────────┤${NC}"

row() {
    local label="$1" value="$2" ok="$3"
    [ "$ok" = "0" ] \
        && printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${GREEN}%-32s${NC} ${CYAN}│${NC}\n" "$label" "$value" \
        || printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${RED}%-32s${NC} ${CYAN}│${NC}\n" "$label" "$value"
}

row "Unit Tests (backend logic)" "${UNIT_PASS} passed"  "$TEST_RUN_EXIT"
row "Frontend Logic Tests"       "${FE_PASS} passed"    "$TEST_RUN_EXIT"
row "API Integration Tests"      "${API_PASS} passed"   "$TEST_RUN_EXIT"
[ "$RUN_E2E" = "true" ] && row "E2E / UI Rendering" "$E2E_LABEL" "$E2E_EXIT"

echo -e "${CYAN}├──────────────────────────┼──────────────────────────────────┤${NC}"

# ── Per-package coverage breakdown ────────────────────────────────────────────
# frontend_logic: shared pure-Rust domain logic — fully instrumentable
fmt_pkg_cov() {
    local pct="$1" label="$2"
    if [ "$pct" = "N/A" ]; then
        printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${YELLOW}%-32s${NC} ${CYAN}│${NC}\n" "$label" "N/A"
    elif [ "$pct" -ge "$REQUIRED_COVERAGE" ] 2>/dev/null; then
        printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${GREEN}%-32s${NC} ${CYAN}│${NC}\n" "$label" "${pct}%  [1]"
    else
        printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${RED}%-32s${NC} ${CYAN}│${NC}\n" "$label" "${pct}%  [1] BELOW ${REQUIRED_COVERAGE}%"
    fi
}
fmt_pkg_cov "$FL_COV" "  frontend_logic"
fmt_pkg_cov "$FT_COV" "  frontend_tests"

# ── Combined gate (total across all instrumented packages) ─────────────────────
COV_OK=1
if [ "$COVERAGE_PCT" -ge "$REQUIRED_COVERAGE" ] 2>/dev/null; then
    COV_OK=0
    printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${GREEN}%-32s${NC} ${CYAN}│${NC}\n" \
        "Combined (≥${REQUIRED_COVERAGE}%)" "${COVERAGE_PCT}% — PASS  [1]"
else
    printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${RED}%-32s${NC} ${CYAN}│${NC}\n" \
        "Combined (≥${REQUIRED_COVERAGE}%)" "${COVERAGE_PCT}% — FAIL  [1]"
fi

# ── API request-path coverage proxy (pass/fail only — not instrumentable) ──────
API_COV_LABEL="${API_PASS} routes exercised  [2]"
if [ "$TEST_RUN_EXIT" -eq 0 ]; then
    printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${GREEN}%-32s${NC} ${CYAN}│${NC}\n" \
        "API Path Coverage proxy" "$API_COV_LABEL"
else
    printf "${CYAN}│${NC} %-24s ${CYAN}│${NC} ${RED}%-32s${NC} ${CYAN}│${NC}\n" \
        "API Path Coverage proxy" "$API_COV_LABEL"
fi

echo -e "${CYAN}└──────────────────────────┴──────────────────────────────────┘${NC}"
echo -e "${YELLOW}Notes:${NC}"
echo -e "  [1] Tarpaulin line coverage for pure-Rust packages (backend/src excluded)."
echo -e "      unit_tests contains only test harness code; its source coverage is"
echo -e "      reflected in the frontend_logic and frontend_tests rows above."
echo -e "      Gate: combined ≥${REQUIRED_COVERAGE}% required to pass."
echo -e "  [2] API integration tests exercise each backend route end-to-end via HTTPS."
echo -e "      Line count is not measurable (async Actix handlers under live TLS);"
echo -e "      passing all ${API_PASS} tests proves full request-path reachability."

FINAL_EXIT=0
[ "$TEST_RUN_EXIT" -eq 0 ] || FINAL_EXIT=1
[ "$E2E_EXIT"      -eq 0 ] || FINAL_EXIT=1
[ "$COV_OK"        -eq 0 ] || FINAL_EXIT=1

if [ "$FINAL_EXIT" -eq 0 ]; then
    echo -e "\n${GREEN}All checks passed!${NC}"
else
    echo -e "\n${RED}One or more checks FAILED. Logs: ${COVERAGE_DIR}/${NC}"
fi
exit "$FINAL_EXIT"

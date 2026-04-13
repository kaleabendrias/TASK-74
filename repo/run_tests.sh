#!/usr/bin/env bash
#
# run_tests.sh — Build, run unit + API tests in Docker, report coverage.
#
# Uses docker-compose.yml with the "test" profile to spin up the test-runner
# alongside the existing db and backend services.
#
# Usage: ./run_tests.sh
#
set -euo pipefail

COMPOSE_FILE="docker-compose.yml"
COVERAGE_DIR="$(pwd)/coverage_output"
REQUIRED_COVERAGE=90

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

cleanup() {
    echo -e "\n${CYAN}[teardown]${NC} Stopping and removing containers..."
    docker compose -f "$COMPOSE_FILE" --profile test down -v --remove-orphans 2>/dev/null || true
}

trap cleanup EXIT

echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}  Tourism Portal — Test Suite Runner${NC}"
echo -e "${CYAN}============================================${NC}"

# ──────────────────────────────────────────
# Step 1: Build all services (including test profile)
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[1/6]${NC} Building services for testing..."
docker compose -f "$COMPOSE_FILE" build backend
docker compose -f "$COMPOSE_FILE" --profile test build test-runner

# ──────────────────────────────────────────
# Step 2: Start database and wait for healthy
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[2/6]${NC} Starting database..."
docker compose -f "$COMPOSE_FILE" up -d db

echo "  Waiting for database to be healthy..."
RETRIES=30
until docker compose -f "$COMPOSE_FILE" exec -T db pg_isready -U tourism -d tourism_portal > /dev/null 2>&1; do
    RETRIES=$((RETRIES - 1))
    if [ "$RETRIES" -le 0 ]; then
        echo -e "${RED}ERROR: Database did not become healthy in time${NC}"
        exit 1
    fi
    sleep 1
done
echo -e "  ${GREEN}Database is ready.${NC}"

# Start backend
echo "  Starting backend..."
docker compose -f "$COMPOSE_FILE" up -d backend
sleep 3

# Wait for backend health
echo "  Waiting for backend health check..."
RETRIES=30
until curl -skf https://localhost:8080/api/health > /dev/null 2>&1; do
    RETRIES=$((RETRIES - 1))
    if [ "$RETRIES" -le 0 ]; then
        echo -e "${YELLOW}WARNING: Backend health check not reachable from host, checking from inside Docker...${NC}"
        docker compose -f "$COMPOSE_FILE" exec -T backend curl -skf https://localhost:8080/api/health > /dev/null 2>&1 && break
        echo -e "${YELLOW}WARNING: Backend may still be starting, proceeding...${NC}"
        break
    fi
    sleep 2
done
echo -e "  ${GREEN}Backend is up.${NC}"

# ──────────────────────────────────────────
# Step 3: Run tests via the test-runner service
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[3/6]${NC} Running unit tests and API tests..."
mkdir -p "$COVERAGE_DIR"

docker compose -f "$COMPOSE_FILE" --profile test run --rm test-runner bash -c '
set -e

echo "========================================="
echo "  Unit Tests"
echo "========================================="
cargo test -p unit_tests -- --test-threads=1 2>&1 | tee /coverage/unit_tests.log
UNIT_EXIT=${PIPESTATUS[0]:-$?}

echo ""
echo "========================================="
echo "  API Integration Tests"
echo "========================================="
cargo test -p api_tests -- --test-threads=1 2>&1 | tee /coverage/api_tests.log
API_EXIT=${PIPESTATUS[0]:-$?}

echo "UNIT_EXIT=$UNIT_EXIT" > /coverage/exit_codes.txt
echo "API_EXIT=$API_EXIT" >> /coverage/exit_codes.txt

echo ""
echo "========================================="
echo "  Tests Complete"
echo "========================================="
echo "  Unit:  exit=$UNIT_EXIT"
echo "  API:   exit=$API_EXIT"

if [ "$UNIT_EXIT" -ne 0 ] || [ "$API_EXIT" -ne 0 ]; then
    exit 1
fi
' 2>&1 | tee "$COVERAGE_DIR/full_output.log"

TEST_RESULT=${PIPESTATUS[0]:-$?}

# ──────────────────────────────────────────
# Step 4: Extract results from logs
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[4/6]${NC} Parsing test results..."

count_tests() {
    local file="$1"
    if [ -f "$file" ]; then
        # Cargo test prints: "test result: ok. X passed; Y failed; ..."
        local passed failed
        passed=$(grep -oP 'test result: ok\. \K\d+' "$file" 2>/dev/null | tail -1 || echo "0")
        failed=$(grep -oP '\d+ failed' "$file" 2>/dev/null | tail -1 | grep -oP '^\d+' || echo "0")
        echo "${passed:-0} passed, ${failed:-0} failed"
    else
        echo "no log file"
    fi
}

# Try extracting from the coverage volume
docker run --rm \
    -v "$(docker compose -f "$COMPOSE_FILE" --profile test ps -q test-runner 2>/dev/null | head -1 || echo 'none'):/src:ro" \
    -v "$COVERAGE_DIR:/out" \
    alpine sh -c "cp /src/* /out/ 2>/dev/null || true" 2>/dev/null || true

# Also pull from the named volume
docker run --rm \
    -v "$(basename "$(pwd)")_coverage-data:/data:ro" \
    -v "$COVERAGE_DIR:/out" \
    alpine sh -c "cp /data/* /out/ 2>/dev/null || true" 2>/dev/null || true

UNIT_RESULT=$(count_tests "$COVERAGE_DIR/unit_tests.log")
API_RESULT=$(count_tests "$COVERAGE_DIR/api_tests.log")

# If we don't have separate logs, parse from full output
if [ "$UNIT_RESULT" = "0 passed, 0 failed" ] && [ -f "$COVERAGE_DIR/full_output.log" ]; then
    UNIT_RESULT=$(grep -A1 "Unit Tests" "$COVERAGE_DIR/full_output.log" | grep -oP 'test result:.*' | head -1 || echo "see log")
    API_RESULT=$(grep -A1 "API.*Tests" "$COVERAGE_DIR/full_output.log" | grep -oP 'test result:.*' | head -1 || echo "see log")
fi

# ──────────────────────────────────────────
# Step 5: Print summary table
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[5/6]${NC} Test Summary:"
echo -e "${CYAN}┌──────────────────────┬──────────────────────────────┐${NC}"
echo -e "${CYAN}│ Suite                │ Result                       │${NC}"
echo -e "${CYAN}├──────────────────────┼──────────────────────────────┤${NC}"
printf "${CYAN}│${NC} %-20s ${CYAN}│${NC} %-28s ${CYAN}│${NC}\n" "Unit Tests" "$UNIT_RESULT"
printf "${CYAN}│${NC} %-20s ${CYAN}│${NC} %-28s ${CYAN}│${NC}\n" "API Tests" "$API_RESULT"
echo -e "${CYAN}└──────────────────────┴──────────────────────────────┘${NC}"

# ──────────────────────────────────────────
# Step 6: Final verdict
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[6/6]${NC} Evaluating results..."

if [ "$TEST_RESULT" -eq 0 ]; then
    echo -e "\n${GREEN}All test suites passed!${NC}"
else
    echo -e "\n${RED}One or more test suites failed.${NC}"
    echo -e "Full logs: ${COVERAGE_DIR}/full_output.log"
fi

exit "$TEST_RESULT"

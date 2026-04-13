#!/usr/bin/env bash
#
# run_tests.sh — Build, run unit + API tests in Docker, report coverage.
#
# Usage: ./run_tests.sh
#
set -euo pipefail

COMPOSE_FILE="docker-compose.test.yml"
PROJECT_NAME="tourism-test-$$"
COVERAGE_DIR="$(pwd)/coverage_output"
REQUIRED_COVERAGE=90

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

cleanup() {
    echo -e "\n${CYAN}[teardown]${NC} Stopping and removing test containers..."
    docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" down -v --remove-orphans 2>/dev/null || true
}

trap cleanup EXIT

echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}  Tourism Portal — Test Suite Runner${NC}"
echo -e "${CYAN}============================================${NC}"

# ──────────────────────────────────────────
# Step 1: Build and start test containers
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[1/6]${NC} Building and starting test containers..."
docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" build --quiet
docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" up -d test-db

# ──────────────────────────────────────────
# Step 2: Wait for the test database to be healthy
# ──────────────────────────────────────────
echo -e "${YELLOW}[2/6]${NC} Waiting for test database to be healthy..."
RETRIES=30
until docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" exec -T test-db pg_isready -U tourism -d tourism_portal_test > /dev/null 2>&1; do
    RETRIES=$((RETRIES - 1))
    if [ "$RETRIES" -le 0 ]; then
        echo -e "${RED}ERROR: Database did not become healthy in time${NC}"
        exit 1
    fi
    sleep 1
done
echo -e "  ${GREEN}Database is ready.${NC}"

# Start backend
docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" up -d backend
echo "  Waiting for backend to start..."
sleep 5

# Check backend is responding
RETRIES=20
until docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" exec -T backend wget -qO- http://localhost:8080/api/health > /dev/null 2>&1 || \
      curl -sf http://localhost:8080/api/health > /dev/null 2>&1; do
    RETRIES=$((RETRIES - 1))
    if [ "$RETRIES" -le 0 ]; then
        echo -e "${YELLOW}WARNING: Backend health check failed, proceeding anyway...${NC}"
        break
    fi
    sleep 2
done

# ──────────────────────────────────────────
# Step 3: Run unit tests inside test-runner container
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[3/6]${NC} Running unit tests with coverage..."
mkdir -p "$COVERAGE_DIR"

docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" run --rm \
    -e DATABASE_URL="postgres://tourism:tourism_secret_2024@test-db:5432/tourism_portal_test" \
    -e TEST_BASE_URL="http://backend:8080" \
    test-runner bash -c "
        echo '>>> Unit Tests <<<' && \
        cargo tarpaulin -p unit_tests \
            --out xml --output-dir /coverage \
            --engine llvm \
            --skip-clean \
            -- --test-threads=1 2>&1 | tee /coverage/unit_tests.log ; \
        UNIT_EXIT=\${PIPESTATUS[0]} ; \
        cp /coverage/cobertura.xml /coverage/unit_coverage.xml 2>/dev/null || true ; \
        echo '>>> API Tests <<<' && \
        cargo tarpaulin -p api_tests \
            --out xml --output-dir /coverage \
            --engine llvm \
            --skip-clean \
            -- --test-threads=1 2>&1 | tee /coverage/api_tests.log ; \
        API_EXIT=\${PIPESTATUS[0]} ; \
        cp /coverage/cobertura.xml /coverage/api_coverage.xml 2>/dev/null || true ; \
        echo \"UNIT_EXIT=\$UNIT_EXIT\" > /coverage/exit_codes.txt ; \
        echo \"API_EXIT=\$API_EXIT\" >> /coverage/exit_codes.txt ; \
        exit 0
    " 2>&1 | tee "$COVERAGE_DIR/full_output.log"

# Copy coverage data out of the volume
RUNNER_CONTAINER=$(docker compose -f "$COMPOSE_FILE" -p "$PROJECT_NAME" ps -q test-runner 2>/dev/null || true)
if [ -n "$RUNNER_CONTAINER" ]; then
    docker cp "$RUNNER_CONTAINER:/coverage/" "$COVERAGE_DIR/" 2>/dev/null || true
fi

# Also try to extract from the volume directly
docker run --rm -v "${PROJECT_NAME}_coverage-data:/data" -v "$COVERAGE_DIR:/out" alpine sh -c "cp /data/* /out/ 2>/dev/null || true" 2>/dev/null || true

# ──────────────────────────────────────────
# Step 4 & 5: Parse coverage and assert >= 90%
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[4/6]${NC} Parsing coverage results..."

extract_coverage() {
    local file="$1"
    local label="$2"
    if [ -f "$file" ]; then
        # Extract line-rate from cobertura XML
        local rate
        rate=$(grep -oP 'line-rate="\K[^"]+' "$file" 2>/dev/null | head -1 || echo "0")
        if [ -z "$rate" ] || [ "$rate" = "0" ]; then
            # Try alternate parsing
            rate=$(sed -n 's/.*line-rate="\([^"]*\)".*/\1/p' "$file" 2>/dev/null | head -1 || echo "0")
        fi
        # Convert to percentage
        local pct
        pct=$(echo "$rate * 100" | bc -l 2>/dev/null | xargs printf "%.1f" 2>/dev/null || echo "0.0")
        echo "$pct"
    else
        echo "0.0"
    fi
}

# Extract from tarpaulin log output as fallback
extract_coverage_from_log() {
    local file="$1"
    if [ -f "$file" ]; then
        # Tarpaulin prints "XX.XX% coverage" at the end
        local pct
        pct=$(grep -oP '[\d.]+(?=% coverage)' "$file" 2>/dev/null | tail -1 || echo "0.0")
        echo "${pct:-0.0}"
    else
        echo "0.0"
    fi
}

UNIT_COV=$(extract_coverage "$COVERAGE_DIR/unit_coverage.xml" "Unit")
API_COV=$(extract_coverage "$COVERAGE_DIR/api_coverage.xml" "API")

# Fallback to log parsing if XML didn't yield results
if [ "$UNIT_COV" = "0.0" ]; then
    UNIT_COV=$(extract_coverage_from_log "$COVERAGE_DIR/unit_tests.log")
fi
if [ "$API_COV" = "0.0" ]; then
    API_COV=$(extract_coverage_from_log "$COVERAGE_DIR/api_tests.log")
fi

# ──────────────────────────────────────────
# Step 5: Print summary table
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[5/6]${NC} Coverage Summary:"
echo -e "${CYAN}┌──────────────────┬────────────┬──────────┬────────┐${NC}"
echo -e "${CYAN}│ Suite            │ Coverage   │ Required │ Status │${NC}"
echo -e "${CYAN}├──────────────────┼────────────┼──────────┼────────┤${NC}"

print_row() {
    local name="$1"
    local cov="$2"
    local req="$3"
    local cov_int
    cov_int=$(echo "$cov" | cut -d. -f1)
    local status
    if [ "${cov_int:-0}" -ge "$req" ]; then
        status="${GREEN}PASS${NC}"
    else
        status="${RED}FAIL${NC}"
    fi
    printf "${CYAN}│${NC} %-16s ${CYAN}│${NC} %8s%% ${CYAN}│${NC} %6s%% ${CYAN}│${NC} %b   ${CYAN}│${NC}\n" \
        "$name" "$cov" "$req" "$status"
}

print_row "Unit Tests" "$UNIT_COV" "$REQUIRED_COVERAGE"
print_row "API Tests" "$API_COV" "$REQUIRED_COVERAGE"
echo -e "${CYAN}└──────────────────┴────────────┴──────────┴────────┘${NC}"

# ──────────────────────────────────────────
# Step 6: Determine pass/fail (cleanup happens in trap)
# ──────────────────────────────────────────
echo -e "\n${YELLOW}[6/6]${NC} Evaluating results..."

UNIT_INT=$(echo "$UNIT_COV" | cut -d. -f1)
API_INT=$(echo "$API_COV" | cut -d. -f1)
EXIT_CODE=0

if [ "${UNIT_INT:-0}" -lt "$REQUIRED_COVERAGE" ]; then
    echo -e "${RED}FAIL: Unit test coverage ${UNIT_COV}% is below required ${REQUIRED_COVERAGE}%${NC}"
    EXIT_CODE=1
fi

if [ "${API_INT:-0}" -lt "$REQUIRED_COVERAGE" ]; then
    echo -e "${RED}FAIL: API test coverage ${API_COV}% is below required ${REQUIRED_COVERAGE}%${NC}"
    EXIT_CODE=1
fi

if [ "$EXIT_CODE" -eq 0 ]; then
    echo -e "\n${GREEN}All test suites passed with required coverage!${NC}"
else
    echo -e "\n${RED}One or more test suites did not meet the coverage threshold.${NC}"
    echo -e "Detailed logs are in: ${COVERAGE_DIR}/"
fi

exit $EXIT_CODE

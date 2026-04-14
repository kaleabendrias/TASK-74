# Regional Tourism Resource & Lodging Operations Portal

A full-stack web application for managing tourism resources, lodging accommodations, inventory, and operational workflows across multiple facilities. The portal provides role-based access for Administrators, Publishers, Reviewers, Clinicians, and Inventory Clerks — supporting the full content lifecycle from draft creation through reviewer approval to publication, lodging management with deposit validation and rent-change negotiation, facility-scoped inventory tracking with near-expiry alerts, bulk Excel import with SSE-streamed progress, watermarked export approvals, HMAC-signed connector integrations, and Prometheus-compatible observability.

## Architecture & Tech Stack

* **Frontend:** Yew (Rust/WASM), compiled with Trunk, served via Nginx
* **Backend:** Actix-web 4 (Rust), Diesel ORM, rustls TLS — mandatory across all profiles
* **Database:** PostgreSQL 16
* **Containerization:** Docker & Docker Compose (Required)

## Project Structure

```text
.
├── backend/                # Actix-web REST API, migrations, background jobs, Dockerfile
├── frontend/               # Yew WASM single-page application, Dockerfile
├── API_tests/              # External HTTP integration test suite (reqwest / tokio)
├── docker-compose.yml      # Multi-container orchestration - MANDATORY
├── run_tests.sh            # Standardized test execution script - MANDATORY
└── README.md               # Project documentation - MANDATORY
```

## Prerequisites

To ensure a consistent environment, this project is designed to run entirely within containers. You must have the following installed:
* [Docker](https://docs.docker.com/get-docker/)
* [Docker Compose](https://docs.docker.com/compose/install/)

## Running the Application

1. **Build and Start Containers:**
   Use Docker Compose to build the images and spin up the entire stack in detached mode.
   ```bash
   docker-compose up --build -d
   ```

2. **Access the App:**
   * Frontend: `https://localhost:8081`
   * Backend API: `https://localhost:8080/api`
   * Health Check (public): `https://localhost:8080/api/health`
   * Readiness Probe (authenticated): `https://localhost:8080/api/health/ready`
   * Prometheus Metrics (Administrator only): `https://localhost:8080/api/metrics`

3. **Stop the Application:**
   ```bash
   docker-compose down -v
   ```

## Testing

All unit, integration, and API tests are executed via a single, standardized shell script. This script automatically handles any necessary container orchestration for the test environment.

Make sure the script is executable, then run it:

```bash
chmod +x run_tests.sh
./run_tests.sh
```

*Note: The `run_tests.sh` script outputs a standard exit code (`0` for success, non-zero for failure) to integrate smoothly with CI/CD validators.*

## Seeded Credentials

The database is pre-seeded with the following test users on startup. All accounts share the password defined by the `INIT_ADMIN_PASSWORD` environment variable.

| Role | Username | Password | Notes |
| :--- | :--- | :--- | :--- |
| **Administrator** | `admin` | *(value of `INIT_ADMIN_PASSWORD`)* | Full access to all system modules including metrics and config. |
| **Publisher** | `publisher` | *(same)* | Creates and submits resources and lodgings; manages rent-change requests. |
| **Reviewer** | `reviewer` | *(same)* | Approves resources and lodgings; issues counterproposals on rent changes. |
| **Clinician** | `clinician` | *(same)* | Read-only access to inventory within assigned facility. |
| **InventoryClerk** | `clerk` | *(same)* | Full inventory management within assigned facility. |

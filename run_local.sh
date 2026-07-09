#!/bin/bash

# Configuration variables
DB_CONTAINER_NAME="fbrustapi-db"
DB_PORT=5432
DB_USER="postgres"
DB_PASSWORD="postgres"
DB_NAME="fbrustapi"
APP_PORT=8080
DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

# Color codes for pretty printing
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0;0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 1. Check/Start Docker daemon / Colima
check_docker() {
    log_info "Checking Docker daemon status..."
    if ! docker ps >/dev/null 2>&1; then
        log_warn "Docker daemon is not running."
        if command -v colima >/dev/null 2>&1; then
            log_info "Colima is installed. Attempting to start Colima..."
            colima start
            
            # Wait for docker to respond
            local max_attempts=15
            local attempt=1
            while [ $attempt -le $max_attempts ]; do
                log_info "Waiting for Docker daemon to become responsive (attempt $attempt/$max_attempts)..."
                if docker ps >/dev/null 2>&1; then
                    log_success "Docker daemon started successfully via Colima."
                    return 0
                fi
                sleep 2
                attempt=$((attempt + 1))
            done
            log_error "Docker daemon failed to start after Colima initialization."
            exit 1
        else
            log_error "Docker daemon is not running and Colima is not installed."
            log_error "Please start Docker Desktop or your Docker daemon manually and try again."
            exit 1
        fi
    else
        log_success "Docker daemon is running."
    fi
}

# 2. Check/Start PostgreSQL Container
setup_db() {
    check_docker
    
    # Check if container exists
    if docker ps -a --format '{{.Names}}' | grep -Eq "^${DB_CONTAINER_NAME}$"; then
        # Check if container is running
        if docker ps --format '{{.Names}}' | grep -Eq "^${DB_CONTAINER_NAME}$"; then
            log_info "PostgreSQL container '${DB_CONTAINER_NAME}' is already running."
        else
            log_info "Starting existing PostgreSQL container '${DB_CONTAINER_NAME}'..."
            docker start "$DB_CONTAINER_NAME" > /dev/null
        fi
    else
        log_info "Creating and starting new PostgreSQL container '${DB_CONTAINER_NAME}'..."
        docker run -d \
            --name "$DB_CONTAINER_NAME" \
            -p "$DB_PORT:5432" \
            -e POSTGRES_PASSWORD="$DB_PASSWORD" \
            -e POSTGRES_USER="$DB_USER" \
            -e POSTGRES_DB="$DB_NAME" \
            postgres:latest > /dev/null
    fi

    # Wait for PostgreSQL to be ready
    log_info "Waiting for PostgreSQL to accept connections..."
    local max_attempts=30
    local attempt=1
    while [ $attempt -le $max_attempts ]; do
        if docker exec "$DB_CONTAINER_NAME" pg_isready -U "$DB_USER" -d "$DB_NAME" >/dev/null 2>&1; then
            log_success "PostgreSQL is ready."
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done

    log_error "Timeout waiting for PostgreSQL to start."
    exit 1
}

# 3. Apply SQL migrations
run_migrations() {
    log_info "Running migrations from migrations/ directory..."
    if [ ! -d "migrations" ]; then
        log_error "migrations/ directory not found!"
        exit 1
    fi

    # We read each sql file in sorted order and pipe it directly to psql in the container
    for file in $(find migrations -name "*.sql" | sort); do
        log_info "Applying migration: $(basename "$file")"
        if ! docker exec -i "$DB_CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -q -a < "$file" >/dev/null 2>&1; then
            log_warn "Migration $(basename "$file") returned an error (it might have already been applied)."
        else
            log_success "Migration $(basename "$file") applied/checked."
        fi
    done
    log_success "Migration process completed."
}

# 4. Clean up / Reset database schema
reset_db() {
    log_info "Resetting database schema..."
    if docker ps --format '{{.Names}}' | grep -Eq "^${DB_CONTAINER_NAME}$"; then
        docker exec -i "$DB_CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" > /dev/null
        log_success "Database schema reset successfully."
    else
        log_warn "PostgreSQL container is not running. Cannot reset DB schema."
    fi
}

# 5. Clean up container entirely
clean_container() {
    log_info "Cleaning up PostgreSQL container '${DB_CONTAINER_NAME}'..."
    docker stop "$DB_CONTAINER_NAME" >/dev/null 2>&1 || true
    docker rm "$DB_CONTAINER_NAME" >/dev/null 2>&1 || true
    log_success "PostgreSQL container removed."
}

# 6. Run Dev environment
run_dev() {
    setup_db
    run_migrations
    
    log_info "Starting fbrustapi in development mode..."
    export DATABASE_URL="$DATABASE_URL"
    cargo run
}

# 7. Run Test suite
run_tests() {
    setup_db
    run_migrations

    log_info "Running Rust unit and integration tests..."
    if ! cargo test; then
        log_error "Some Rust unit/integration tests failed! Aborting."
        exit 1
    fi
    log_success "All Rust unit/integration tests passed."

    log_info "Building fbrustapi..."
    cargo build
    
    log_info "Starting fbrustapi in background for testing..."
    export DATABASE_URL="$DATABASE_URL"
    mkdir -p target
    cargo run > target/fbrustapi_test.log 2>&1 &
    local app_pid=$!
    
    # Ensure background process is killed on exit
    trap 'log_info "Stopping application (PID $app_pid)..."; kill $app_pid 2>/dev/null; wait $app_pid 2>/dev/null; log_success "Application stopped."' EXIT INT TERM

    # Wait for server to respond
    log_info "Waiting for application to start on port $APP_PORT..."
    local max_attempts=15
    local attempt=1
    local server_ready=false
    while [ $attempt -le $max_attempts ]; do
        local response
        response=$(curl -s http://localhost:${APP_PORT}/health)
        if [ "$response" = "ok" ]; then
            server_ready=true
            break
        fi
        sleep 1
        attempt=$((attempt + 1))
    done

    if [ "$server_ready" = "false" ]; then
        log_error "Application failed to start or respond to health check."
        log_error "Logs from execution:"
        cat target/fbrustapi_test.log
        exit 1
    fi

    log_success "Application is online!"
    
    # Run tests
    log_info "Running local health check integration test..."
    local health_code
    health_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:${APP_PORT}/health)
    
    if [ "$health_code" = "200" ]; then
        log_success "TEST PASSED: GET /health returned HTTP 200"
    else
        log_error "TEST FAILED: GET /health returned HTTP $health_code"
        exit 1
    fi

    # Additional endpoints could be tested here as they get implemented
    log_success "All local tests completed successfully!"
}

# Help / Usage instruction
print_usage() {
    echo "Usage: $0 {dev|test|clean|reset|help}"
    echo
    echo "Commands:"
    echo "  dev      - Starts PostgreSQL (using Docker/Colima), runs migrations, and starts the API in foreground."
    echo "  test     - Starts PostgreSQL, runs migrations, starts the API in background, runs health verification tests, and stops."
    echo "  clean    - Stops and removes the local PostgreSQL Docker container."
    echo "  reset    - Resets the database schema (drops and recreates the public schema) and reapplies migrations."
    echo "  help     - Shows this help message."
}

# Command dispatching
case "$1" in
    dev)
        run_dev
        ;;
    test)
        run_tests
        ;;
    clean)
        clean_container
        ;;
    reset)
        setup_db
        reset_db
        run_migrations
        ;;
    help|*)
        print_usage
        ;;
esac

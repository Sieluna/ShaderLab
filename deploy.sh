#!/bin/bash
#
# Manages installation, deployment, rollback for Senra Server
#
# Usage: ./deploy.sh [options] <command>
# See show_usage() for detailed usage information
#
# Copyright (c) 2025 Sieluna
# Licensed under the MIT License

set -euo pipefail
IFS=$'\n\t'

#######################################
# Configuration Constants
#######################################
readonly DEFAULT_GITHUB_REPO="Sieluna/ShaderLab"
readonly DEFAULT_BINARY_NAME="senra_server-x86_64-unknown-linux-gnu"
readonly DEFAULT_SERVICE_NAME="senra_server"
readonly DEFAULT_APP_BASE_DIR="/opt/senra"
readonly DEFAULT_DATA_DIR="/var/lib/senra"
readonly DEFAULT_SERVICE_USER="senra"
readonly DEFAULT_SERVICE_GROUP="senra"
readonly DEFAULT_HEALTH_CHECK_TIMEOUT="30"
readonly DEFAULT_HEALTH_CHECK_ENDPOINT="/health"

# Configuration variables (can be overridden by environment)
GITHUB_REPO="${GITHUB_REPO:-${DEFAULT_GITHUB_REPO}}"
BINARY_NAME="${BINARY_NAME:-${DEFAULT_BINARY_NAME}}"
SERVICE_NAME="${SERVICE_NAME:-${DEFAULT_SERVICE_NAME}}"
APP_BASE_DIR="${APP_BASE_DIR:-${DEFAULT_APP_BASE_DIR}}"
DATA_DIR="${DATA_DIR:-${DEFAULT_DATA_DIR}}"
SERVICE_USER="${SERVICE_USER:-${DEFAULT_SERVICE_USER}}"
SERVICE_GROUP="${SERVICE_GROUP:-${DEFAULT_SERVICE_GROUP}}"
HEALTH_CHECK_TIMEOUT="${HEALTH_CHECK_TIMEOUT:-${DEFAULT_HEALTH_CHECK_TIMEOUT}}"
HEALTH_CHECK_ENDPOINT="${HEALTH_CHECK_ENDPOINT:-${DEFAULT_HEALTH_CHECK_ENDPOINT}}"

# Derived paths
RELEASES_DIR="${APP_BASE_DIR}/releases"
CURRENT_SYMLINK="${APP_BASE_DIR}/current"
DB_FILE="${DATA_DIR}/shaderlab.db"
LOG_FILE="/var/log/${SERVICE_NAME}/deploy.log"
LOG_DIR="/var/log/${SERVICE_NAME}"

#######################################
# Utility Functions
#######################################

# Logs a message to stdio and a log file. Exits on ERROR.
# Arguments:
#   $1: Log level (INFO, WARN, ERROR)
#   $2: Log message
log() {
	local -r level="${1}"
	local -r message="${2}"
	local -r timestamp="$(date '+%Y-%m-%d %H:%M:%S')"

	echo "[${timestamp}] [${level}] ${message}" | tee -a "${LOG_FILE:-/dev/null}" >&2
}

log_info() {
	log "INFO" "${1}"
}

log_warn() {
	log "WARN" "${1}"
}

log_error() {
	log "ERROR" "${1}"
	exit 1
}

# Verifies the script is running as the root user.
check_root() {
	if [[ "$(id -u)" -ne 0 ]]; then
		log_error "This script requires root privileges. Use 'sudo ${0} <command>'"
	fi
}

# Ensures a directory exists with specified ownership and permissions.
# Arguments:
#   $1: Directory path
#   $2: Owner:group (optional)
#   $3: Permissions (optional, defaults to 755)
ensure_directory() {
	local -r directory="${1}"
	local -r owner="${2:-}"
	local -r permissions="${3:-755}"

	if [[ ! -d "${directory}" ]]; then
		log_info "Creating directory: ${directory}"
		if ! mkdir -p "${directory}"; then
			log_error "Failed to create directory: ${directory}"
		fi
	fi

	if [[ -n "${owner}" ]]; then
		if ! chown "${owner}" "${directory}"; then
			log_error "Failed to set ownership on directory: ${directory}"
		fi
	fi

	if ! chmod "${permissions}" "${directory}"; then
		log_error "Failed to set permissions on directory: ${directory}"
	fi
}

# Checks and installs required system dependencies
ensure_dependencies() {
	local -A pkg_map=(
		["curl"]="curl curl curl"
		["jq"]="jq jq jq"
		["setcap"]="libcap2-bin libcap libcap"
		["sha256sum"]="coreutils coreutils coreutils"
		["sqlite3"]="sqlite3 sqlite sqlite"
		["unzip"]="unzip unzip unzip"
	)
	local -a missing_pkgs=()
	local pkg_cmd idx=0

	# Detect package manager
	if command -v apt-get > /dev/null 2>&1; then
		pkg_cmd="apt-get" idx=1
	elif command -v yum > /dev/null 2>&1; then
		pkg_cmd="yum" idx=2
	elif command -v dnf > /dev/null 2>&1; then
		pkg_cmd="dnf" idx=3
	else
		log_error "Unsupported package manager (need apt-get, yum or dnf)"
	fi

	# Find missing packages
	for cmd in "${!pkg_map[@]}"; do
		if ! command -v "${cmd}" > /dev/null 2>&1; then
			IFS=' ' read -ra pkg_parts <<< "${pkg_map[$cmd]}"
			missing_pkgs+=("${pkg_parts[$((idx - 1))]}")
		fi
	done

	# Install missing packages if any
	if ((${#missing_pkgs[@]} > 0)); then
		log_info "Installing: ${missing_pkgs[*]}"
		[[ "$pkg_cmd" == "apt-get" ]] && sudo apt-get update
		${pkg_cmd} install -y "${missing_pkgs[@]}" || log_error "Failed to install packages"
	fi
}

# Creates the service user and group if they do not exist.
create_service_user() {
	if ! getent group "${SERVICE_GROUP}" > /dev/null; then
		log_info "Creating service group: ${SERVICE_GROUP}"
		if ! groupadd --system "${SERVICE_GROUP}"; then
			log_error "Failed to create service group: ${SERVICE_GROUP}"
		fi
	fi

	if ! id "${SERVICE_USER}" > /dev/null 2>&1; then
		log_info "Creating service user: ${SERVICE_USER}"
		if ! useradd --system --shell /bin/false --home-dir "${APP_BASE_DIR}" \
			--create-home --gid "${SERVICE_GROUP}" "${SERVICE_USER}"; then
			log_error "Failed to create service user: ${SERVICE_USER}"
		fi
	fi
}

# Configures Nginx as reverse proxy for port 80 -> 3000
setup_nginx() {
	log_info "Setting up Nginx reverse proxy..."

	# Install Nginx if not present
	if ! command -v nginx > /dev/null 2>&1; then
		log_info "Installing Nginx..."
		apt-get update > /dev/null 2>&1 || log_error "Failed to update package list"
		apt-get install -y nginx > /dev/null 2>&1 || log_error "Failed to install Nginx"
	fi

	# Ensure sites-available directory exists
	mkdir -p /etc/nginx/sites-available /etc/nginx/sites-enabled

	# Create Nginx configuration
	cat > /etc/nginx/sites-available/senra << 'NGINX_EOF'
server {
    listen 80 default_server;
    server_name _;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
    }
}
NGINX_EOF

	# Enable site
	if [[ ! -L /etc/nginx/sites-enabled/senra ]]; then
		ln -s /etc/nginx/sites-available/senra /etc/nginx/sites-enabled/senra
	fi

	# Remove default site
	rm -f /etc/nginx/sites-enabled/default

	# Test configuration
	if ! nginx -t > /dev/null 2>&1; then
		log_error "Nginx configuration test failed"
	fi

	# Enable and start Nginx
	systemctl enable nginx > /dev/null 2>&1 || log_error "Failed to enable Nginx"
	systemctl restart nginx > /dev/null 2>&1 || log_error "Failed to restart Nginx"

	log_info "Nginx reverse proxy configured successfully"
}

# Sets secure, consistent permissions on application directories and files.
set_secure_permissions() {
	local -r owner="${SERVICE_USER}:${SERVICE_GROUP}"

	log_info "Setting secure permissions..."
	ensure_directory "${APP_BASE_DIR}" "${owner}" 755
	ensure_directory "${RELEASES_DIR}" "${owner}" 755
	ensure_directory "${DATA_DIR}" "${owner}" 755
	ensure_directory "${LOG_DIR}" "${owner}" 755

	# Database file: mode=644
	if [[ -f "${DB_FILE}" ]]; then
		if ! chown "${owner}" "${DB_FILE}"; then
			log_error "Failed to set ownership on ${DB_FILE}"
		fi
		if ! chmod 644 "${DB_FILE}"; then
			log_error "Failed to set permissions on ${DB_FILE}"
		fi
	fi
}

# Performs an application health check via an HTTP endpoint.
# Arguments:
#   $1: Health check endpoint (default: HEALTH_CHECK_ENDPOINT)
#   $2: Timeout in seconds (default: HEALTH_CHECK_TIMEOUT)
#   $3: Port number (default: 80)
# Returns:
#   0 if healthy, 1 if unhealthy
health_check() {
	local -r endpoint="${1:-${HEALTH_CHECK_ENDPOINT}}"
	local -r timeout="${2:-${HEALTH_CHECK_TIMEOUT}}"
	local -r port="${3:-80}"
	local i

	log_info "Performing health check (timeout: ${timeout}s)..."

	for ((i = 1; i <= timeout; i++)); do
		if curl --fail --silent --max-time 5 "http://localhost:${port}${endpoint}" > /dev/null; then
			log_info "Health check passed after ${i}s."
			return 0
		fi

		if ! systemctl is-active --quiet "${SERVICE_NAME}"; then
			log_error "Service stopped during health check"
			return 1
		fi

		((i % 10 == 0)) && log_info "Health check still running... (${i}/${timeout}s)"

		sleep 1
	done

	log_error "Health check failed after ${timeout}s"
	return 1
}

# Verifies file integrity using SHA256 checksum.
# Arguments:
#   $1: Path to file to verify
#   $2: Expected SHA256 checksum
# Returns:
#   0 if checksum matches, 1 otherwise
verify_integrity() {
	local -r file_path="${1}"
	local expected_checksum="${2}"
	local actual_checksum

	if [[ ! -f "${file_path}" ]]; then
		log_error "File does not exist: ${file_path}"
		return 1
	fi

	if [[ -z "${expected_checksum}" ]]; then
		log_warn "No checksum provided for verification"
		return 0
	fi

	log_info "Verifying file integrity..."
	IFS=' ' read -r actual_checksum _ < <(sha256sum "${file_path}")

	if [[ "${actual_checksum}" != "${expected_checksum}" ]]; then
		log_error "File integrity check failed. Expected: ${expected_checksum}, Actual: ${actual_checksum}"
		return 1
	fi

	log_info "File integrity verified successfully"
	return 0
}

# Downloads a file from a URL with retry.
# Arguments:
#   $1: URL to download
#   $2: Output file path
# Returns:
#   0 if successful, 1 if all retries failed
download() {
	local -r url="$1" output="$2"

	log_info "Downloading from ${url} to ${output}..."
	if curl --location --fail --retry 3 --retry-delay 5 \
		--connect-timeout 20 --max-time 300 "${url}" -o "${output}"; then
		log_info "Download successful"
		return 0
	fi

	log_error "Download failed after multiple retries."
	return 1
}

#######################################
# Service Management Functions
#######################################

# Starts the systemd service.
# Returns:
#   0 if service started successfully, 1 otherwise.
start_service() {
	log_info "Starting service: ${SERVICE_NAME}"
	if ! systemctl start "${SERVICE_NAME}"; then
		log_error "Failed to start service: ${SERVICE_NAME}"
		return 1
	fi
	return 0
}

# Stops the systemd service.
# Returns:
#   0 if service stopped successfully, 1 otherwise.
stop_service() {
	if ! is_service_running; then
		log_info "Service ${SERVICE_NAME} is not running, no need to stop."
		return 0
	fi
	log_info "Stopping service: ${SERVICE_NAME}"
	if ! systemctl stop "${SERVICE_NAME}"; then
		log_error "Failed to stop service: ${SERVICE_NAME}"
		return 1
	fi
	return 0
}

# Restarts the systemd service.
# Returns:
#   0 if service restarted successfully, 1 otherwise.
restart_service() {
	log_info "Restarting service: ${SERVICE_NAME}"
	if ! systemctl restart "${SERVICE_NAME}"; then
		log_error "Failed to restart service: ${SERVICE_NAME}"
		return 1
	fi
	return 0
}

# Waits for the service to become active.
# Arguments:
#   $1: Timeout in seconds (default: 30)
# Returns:
#   0 if service is active within timeout, 1 otherwise.
wait_for_service_active() {
	local -r timeout="${1:-30}"
	local i

	log_info "Waiting for service to become active (timeout: ${timeout}s)..."
	for ((i = 1; i <= timeout; i++)); do
		if systemctl is-active --quiet "${SERVICE_NAME}"; then
			log_info "Service is now active."
			return 0
		fi
		sleep 1
	done

	log_error "Service did not become active within ${timeout}s."
	systemctl status "${SERVICE_NAME}" --no-pager >&2
	return 1
}

# Checks if the systemd service is currently running
# Returns:
#   0 if service is active, 1 otherwise
is_service_running() {
	systemctl is-active --quiet "${SERVICE_NAME}" 2> /dev/null
}

# Fetches release information from GitHub API.
# Arguments:
#   $1: Optional release tag/version. If empty, fetches the latest release.
# Returns:
#   0 on success, 1 on failure.
# Output:
#   Prints the download URL and SHA256 digest to standard output on success.
get_release() {
	local tag="${1:-}"
	local api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/${tag:+tags/}${tag:-latest}"
	local release_info

	log_info "Fetching release information${tag:+ for version: ${tag}}..."

	release_info=$(curl -sS --fail-with-body "${api_url}" 2>&1) || {
		log_error "Failed to fetch release from GitHub. Check URL: ${api_url}. Error: ${release_info}"
		return 1
	}

	if echo "${release_info}" | jq -e '.message' > /dev/null; then
		log_error "GitHub API returned an error: $(echo "${release_info}" | jq -r '.message')"
		return 1
	fi

	local -a result
	mapfile -t result < <(echo "${release_info}" \
		| jq -r --arg binary_name "${BINARY_NAME}" '
			.assets[] | select(.name | contains($binary_name + ".zip")) |
			.browser_download_url, (.digest // "" | sub("^sha256:"; ""))
		')

	if [[ $? -ne 0 || -z "${result[0]}" ]]; then
		log_error "Asset '${BINARY_NAME}.zip' not found"
		return 1
	fi

	echo "${result[0]}"
	[[ -n "${result[1]}" ]] && echo "${result[1]}"

	log_info "Found release: ${result[0]}"
	return 0
}

#######################################
# Command Implementations
#######################################

cmd_install() {
	local version="${1:-}"

	check_root
	log_info "Initializing environment..."

	ensure_directory "${LOG_DIR}" "" "755"
	ensure_dependencies
	create_service_user

	if [ -f "${DB_FILE}" ]; then
		sudo -u "${SERVICE_USER}" sqlite3 "${DB_FILE}" "VACUUM;"
	fi

	ensure_directory "${APP_BASE_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"
	ensure_directory "${RELEASES_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"
	ensure_directory "${DATA_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"

	set_secure_permissions

	# Create empty SQLite database file
	if [[ ! -f "${DB_FILE}" ]]; then
		log_info "Creating empty SQLite database file..."
		touch "${DB_FILE}"
		chown "${SERVICE_USER}:${SERVICE_GROUP}" "${DB_FILE}"
		chmod 644 "${DB_FILE}"
	fi

	# Write systemd unit atomically
	cat << EOF > "/etc/systemd/system/${SERVICE_NAME}.service"
[Unit]
Description=Senra Server
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
ExecStart=${CURRENT_SYMLINK}/${BINARY_NAME}
WorkingDirectory=${CURRENT_SYMLINK}
Restart=on-failure
RestartSec=5s
User=${SERVICE_USER}
Group=${SERVICE_GROUP}

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectHome=true
ReadWritePaths=${APP_BASE_DIR} ${DATA_DIR} ${LOG_DIR}

# Environment variables
Environment="HOST=0.0.0.0"
Environment="PORT=3000"
Environment="DATABASE_URL=sqlite:file:${DB_FILE}"
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
EOF

	if ! systemctl daemon-reload; then
		log_error "Failed to reload systemd daemon"
	fi

	if ! systemctl enable "${SERVICE_NAME}"; then
		log_error "Failed to enable service ${SERVICE_NAME}"
	fi

	log_info "Environment initialized successfully."

	# Setup Nginx reverse proxy
	setup_nginx

	if [[ -n "${version}" ]]; then
		cmd_deploy "${version}"
	else
		cmd_deploy
	fi
}

cmd_deploy() {
	local version="${1:-}"

	check_root
	log_info "Starting deployment ${version:+of version: $version}..."

	ensure_directory "${RELEASES_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"

	local -a release_data
	mapfile -t release_data < <(get_release "${version}")
	local -r asset_url="${release_data[0]}"
	local -r sha256_digest="${release_data[1]:-}"

	if [[ -z "${asset_url}" || "${asset_url}" == "null" ]]; then
		log_error "Release asset not found"
	fi

	# Create temporary directory with proper cleanup
	local temp_dir
	if ! temp_dir=$(mktemp -d -p "${RELEASES_DIR}" deploy.XXXXXXXXXX); then
		log_error "Failed to create temporary directory"
	fi

	# Ensure cleanup on exit - only delete if directory still exists
	trap '[[ -d '"${temp_dir}"' ]] && rm -rf '"${temp_dir}"'' EXIT

	# Download release
	download "${asset_url}" "${temp_dir}/release.zip"

	# Verify checksum if available from GitHub API
	if [[ -n "${sha256_digest}" ]]; then
		verify_integrity "${temp_dir}/release.zip" "${sha256_digest}"
	fi

	# Extract and validate
	if ! unzip -o "${temp_dir}/release.zip" -d "${temp_dir}"; then
		log_error "Failed to extract release archive"
	fi
	rm "${temp_dir}/release.zip"

	if [[ ! -f "${temp_dir}/${BINARY_NAME}" ]]; then
		log_error "Binary '${BINARY_NAME}' not found in release package"
	fi

	# Make binary executable
	if ! chmod +x "${temp_dir}/${BINARY_NAME}"; then
		log_error "Failed to make binary executable"
	fi

	# Rename to timestamped directory
	local release_dir
	release_dir="${RELEASES_DIR}/$(date +%Y%m%d%H%M%S%N)"
	if ! mv "${temp_dir}" "${release_dir}"; then
		log_error "Failed to move release to final directory"
	fi

	# Set proper ownership
	if ! chown -R "${SERVICE_USER}:${SERVICE_GROUP}" "${release_dir}"; then
		log_error "Failed to set ownership on release directory"
	fi

	# Set capability to bind to port
	if ! setcap 'cap_net_bind_service=+ep' "${release_dir}/${BINARY_NAME}"; then
		log_error "Failed to set capability to bind to port"
	fi

	# Update symlink
	if ! ln -sfn "${release_dir}" "${CURRENT_SYMLINK}"; then
		log_error "Failed to update current symlink"
	fi

	# Restart service and verify
	if restart_service; then
		log_info "Deployment successful: $(basename "${release_dir}")"
	else
		log_error "Deployment failed during service restart"
	fi
}

cmd_rollback() {
	local version="${1:-}"
	check_root
	log_info "Starting rollback to previous version..."

	# Ensure releases directory exists
	if [[ ! -d "${RELEASES_DIR}" ]]; then
		log_error "Releases directory ${RELEASES_DIR} does not exist"
	fi

	# If specific version is provided, rollback to that version
	if [[ -n "${version}" ]]; then
		local target_release="${RELEASES_DIR}/${version}"
		if [[ ! -d "${target_release}" ]]; then
			log_error "Version ${version} not found in releases directory"
		fi

		log_info "Rolling back to specified version: ${version}"
		if ! ln -sfn "${target_release}" "${CURRENT_SYMLINK}"; then
			log_error "Failed to update symlink for rollback to ${version}"
		fi

		if restart_service; then
			log_info "Rollback successful: ${version}"
		else
			log_error "Rollback failed during service restart"
		fi
		return 0
	fi

	# Get sorted list of valid releases
	local -a releases=()
	while IFS= read -r -d $'\0'; do
		local base
		base=$(basename "${REPLY}")
		if [[ "${base}" =~ ^[0-9]{14,}$ ]]; then
			releases+=("${REPLY}")
		fi
	done < <(find "${RELEASES_DIR}" -maxdepth 1 -mindepth 1 -type d -print0 2> /dev/null | sort -z)

	if [[ ${#releases[@]} -lt 2 ]]; then
		log_error "No previous version available for rollback"
	fi

	local -r previous_release="${releases[-2]}"
	log_info "Rolling back to: $(basename "${previous_release}")"

	# Update symlink
	if ! ln -sfn "${previous_release}" "${CURRENT_SYMLINK}"; then
		log_error "Failed to update symlink for rollback"
	fi

	# Restart service and verify
	if restart_service; then
		log_info "Rollback successful: $(basename "${previous_release}")"
	else
		log_error "Rollback failed during service restart"
	fi
}

cmd_uninstall() {
	local version="${1:-}"

	check_root

	if [[ -n "${version}" ]]; then
		log_info "Uninstalling specific version: ${version}"
		local target_release="${RELEASES_DIR}/${version}"
		if [[ ! -d "${target_release}" ]]; then
			log_error "Version ${version} not found in releases directory"
		fi

		if [[ -L "${CURRENT_SYMLINK}" ]] && [[ "$(readlink "${CURRENT_SYMLINK}")" == "${target_release}" ]]; then
			log_info "Stopping service as current version is being uninstalled"
			stop_service || log_warn "Failed to stop service gracefully"
			rm -f "${CURRENT_SYMLINK}"
		fi

		log_info "Removing version: ${version}"
		rm -rf "${target_release}"
		log_info "Version ${version} uninstalled successfully"
		return 0
	fi

	# Full uninstall
	log_info "Starting complete uninstallation..."

	# Stop and disable service
	if systemctl list-unit-files --type=service | grep -q "^${SERVICE_NAME}.service"; then
		log_info "Stopping and disabling service: ${SERVICE_NAME}"
		systemctl stop "${SERVICE_NAME}" 2> /dev/null || true
		systemctl disable "${SERVICE_NAME}" 2> /dev/null || true
		rm -f "/etc/systemd/system/${SERVICE_NAME}.service"
		systemctl daemon-reload
	fi

	# Cleanup Nginx configuration
	if [[ -f /etc/nginx/sites-available/senra ]]; then
		log_info "Removing Nginx reverse proxy configuration..."
		rm -f /etc/nginx/sites-available/senra
		rm -f /etc/nginx/sites-enabled/senra
		systemctl restart nginx 2> /dev/null || true
	fi

	# Remove application directories
	log_info "Removing application directories..."
	rm -rf "${APP_BASE_DIR}"
	rm -rf "${DATA_DIR}"
	rm -rf "${LOG_DIR}"

	log_warn "Service user '${SERVICE_USER}' and group '${SERVICE_GROUP}' were not removed"
	log_info "Complete uninstallation finished"
}

cmd_list() {
	echo "=== Available Releases ==="

	if [[ ! -d "${RELEASES_DIR}" ]]; then
		echo "No releases directory found at ${RELEASES_DIR}"
		return 0
	fi

	# List releases with current marking
	if [[ -L "${CURRENT_SYMLINK}" ]]; then
		local current_release
		current_release=$(basename "$(readlink "${CURRENT_SYMLINK}")")
		find "${RELEASES_DIR}" -maxdepth 1 -mindepth 1 -type d -printf '%f\n' | sort -r | while read -r release; do
			if [[ "${release}" == "${current_release}" ]]; then
				echo "  ${release} (current)"
			else
				echo "  ${release}"
			fi
		done
	else
		# No current symlink, just list all
		find "${RELEASES_DIR}" -maxdepth 1 -mindepth 1 -type d -printf '%f\n' | sort -r | while read -r release; do
			echo "  ${release}"
		done
	fi
}

cmd_start() {
	check_root

	# Verify service is installed
	if ! systemctl list-unit-files --type=service | grep -q "^${SERVICE_NAME}.service"; then
		log_error "Service ${SERVICE_NAME} is not installed"
	fi

	# Start service
	if ! start_service; then
		return 1
	fi

	log_info "Service started successfully"
}

cmd_stop() {
	check_root

	# Verify service is installed
	if ! systemctl list-unit-files --type=service | grep -q "^${SERVICE_NAME}.service"; then
		log_error "Service ${SERVICE_NAME} is not installed"
	fi

	# Stop service
	if ! stop_service; then
		return 1
	fi

	log_info "Service stopped successfully"
}

cmd_status() {
	echo "=== Senra Server Status ==="
	echo

	# Service status
	echo "--- Service Status ---"
	if systemctl list-unit-files --type=service | grep -q "^${SERVICE_NAME}.service"; then
		if is_service_running; then
			echo "Status: RUNNING"
			systemctl status "${SERVICE_NAME}" --no-pager --lines=5 2> /dev/null || true
		else
			echo "Status: STOPPED"
			systemctl status "${SERVICE_NAME}" --no-pager --lines=3 2> /dev/null || true
		fi
	else
		echo "Status: NOT INSTALLED"
	fi

	echo
	echo "--- Current Release ---"
	if [[ -L "${CURRENT_SYMLINK}" ]]; then
		local current_target
		current_target=$(readlink "${CURRENT_SYMLINK}")
		if [[ -d "${current_target}" ]]; then
			local current_release
			current_release=$(basename "${current_target}")
			echo "Current: ${current_release}"

			# Show binary info if available
			local binary_path="${current_target}/${BINARY_NAME}"
			if [[ -f "${binary_path}" ]]; then
				echo "Binary: ${binary_path}"
				echo "Size: $(du -h "${binary_path}" | cut -f1)"
				echo "Modified: $(stat -c '%y' "${binary_path}" | cut -d'.' -f1)"
			fi
		else
			echo "ERROR: Current symlink points to non-existent directory: ${current_target}"
		fi
	else
		echo "No current release configured"
	fi

	echo
	echo "--- Health Check ---"
	if systemctl list-unit-files --type=service | grep -q "^${SERVICE_NAME}.service" && is_service_running; then
		if health_check "${HEALTH_CHECK_ENDPOINT}" 10 80; then
			echo "Health: HEALTHY ✓"
		else
			echo "Health: UNHEALTHY ✗"
		fi
	else
		echo "Health: SERVICE NOT RUNNING"
	fi

	echo
	echo "--- Configuration ---"
	echo "Service Name: ${SERVICE_NAME}"
	echo "Service User: ${SERVICE_USER}"
	echo "App Directory: ${APP_BASE_DIR}"
	echo "Data Directory: ${DATA_DIR}"
	echo "Log Directory: ${LOG_DIR}"
	echo "Database: ${DB_FILE}"

	# Show disk usage
	echo
	echo "--- Disk Usage ---"
	if [[ -d "${APP_BASE_DIR}" ]]; then
		echo "App Directory: $(du -sh "${APP_BASE_DIR}" 2> /dev/null | cut -f1 || echo "N/A")"
	fi
	if [[ -d "${DATA_DIR}" ]]; then
		echo "Data Directory: $(du -sh "${DATA_DIR}" 2> /dev/null | cut -f1 || echo "N/A")"
	fi
	if [[ -f "${DB_FILE}" ]]; then
		echo "Database File: $(du -sh "${DB_FILE}" 2> /dev/null | cut -f1 || echo "N/A")"
	fi
}

cmd_usage() {
	cat << EOF
Senra Server Management Tool

Usage: sudo ${0} [options] <command>

Options:
  --repo=<repo>           GitHub repository (default: ${DEFAULT_GITHUB_REPO})
  --binary=<name>         Binary name (default: ${DEFAULT_BINARY_NAME})
  --service=<name>        Service name (default: ${DEFAULT_SERVICE_NAME})
  --app-dir=<path>        Application base directory (default: ${DEFAULT_APP_BASE_DIR})
  --data-dir=<path>       Data directory (default: ${DEFAULT_DATA_DIR})
  --user=<user>           Service user (default: ${DEFAULT_SERVICE_USER})
  --group=<group>         Service group (default: ${DEFAULT_SERVICE_GROUP})

Commands:
  install [VERSION]       Initialize environment and install service (optionally specific version)
  rollback [VERSION]      Rollback to previous version or specific version
  uninstall [VERSION]     Uninstall completely or remove specific version
  list                    List available releases
  start                   Start the service
  stop                    Stop the service
  restart                 Restart the service (same as deploy to current version)
  status                  Show service status and health
  help                    Show this help

Examples:
  # Standard installation
  sudo ${0} install
  sudo ${0} deploy

  # Custom configuration
  sudo ${0} --user=myuser --data-dir=/custom/data install
EOF
}

# =========================
# Main Entry
# =========================

parse_arguments() {
	local arg
	local i=1
	local argc=$#

	# First pass: extract options and positional arguments
	while ((i <= argc)); do
		arg="${!i}"

		case "${arg}" in
			# Handle --help and -h
			-h | --help)
				cmd_usage
				exit 0
				;;

			# Handle long options with = syntax (--option=value)
			--repo=*)
				GITHUB_REPO="${arg#--repo=}"
				[[ -z "${GITHUB_REPO}" ]] && log_error "--repo requires a non-empty value"
				;;
			--binary=*)
				BINARY_NAME="${arg#--binary=}"
				[[ -z "${BINARY_NAME}" ]] && log_error "--binary requires a non-empty value"
				;;
			--service=*)
				SERVICE_NAME="${arg#--service=}"
				[[ -z "${SERVICE_NAME}" ]] && log_error "--service requires a non-empty value"
				;;
			--app-dir=*)
				APP_BASE_DIR="${arg#--app-dir=}"
				[[ -z "${APP_BASE_DIR}" ]] && log_error "--app-dir requires a non-empty value"
				;;
			--data-dir=*)
				DATA_DIR="${arg#--data-dir=}"
				[[ -z "${DATA_DIR}" ]] && log_error "--data-dir requires a non-empty value"
				;;
			--user=*)
				SERVICE_USER="${arg#--user=}"
				[[ -z "${SERVICE_USER}" ]] && log_error "--user requires a non-empty value"
				;;
			--group=*)
				SERVICE_GROUP="${arg#--group=}"
				[[ -z "${SERVICE_GROUP}" ]] && log_error "--group requires a non-empty value"
				;;

			# Handle long options with space syntax (--option value)
			--repo | --binary | --service | --app-dir | --data-dir | --user | --group)
				((i++))
				if ((i > argc)); then
					log_error "${arg} requires a value"
				fi

				local value="${!i}"
				case "${arg}" in
					--repo) GITHUB_REPO="${value}" ;;
					--binary) BINARY_NAME="${value}" ;;
					--service) SERVICE_NAME="${value}" ;;
					--app-dir) APP_BASE_DIR="${value}" ;;
					--data-dir) DATA_DIR="${value}" ;;
					--user) SERVICE_USER="${value}" ;;
					--group) SERVICE_GROUP="${value}" ;;
				esac

				[[ -z "${value}" ]] && log_error "${arg} requires a non-empty value"
				;;

			# Handle unknown options
			--*)
				log_error "Unknown option: ${arg}"
				;;

			-*)
				log_error "Unknown option: ${arg}"
				;;

			# Positional arguments (command and optional version)
			*)
				COMMAND_ARGS+=("${arg}")
				;;
		esac

		((i++))
	done

	# Recalculate dependent paths after parsing
	RELEASES_DIR="${APP_BASE_DIR}/releases"
	CURRENT_SYMLINK="${APP_BASE_DIR}/current"
	DB_FILE="${DATA_DIR}/shaderlab.db"
	LOG_DIR="/var/log/${SERVICE_NAME}"
	LOG_FILE="${LOG_DIR}/deploy.log"
}

main() {
	# Initialize command args array
	COMMAND_ARGS=()

	# Parse all arguments
	parse_arguments "$@"

	# Ensure log directory exists if we're root
	if [[ "$(id -u)" -eq 0 ]] && [[ ! -d "${LOG_DIR}" ]]; then
		if ! mkdir -p "${LOG_DIR}"; then
			echo "Warning: Failed to create log directory ${LOG_DIR}" >&2
		fi
	fi

	# Extract command and version from positional arguments
	local command="${COMMAND_ARGS[0]:-help}"
	local version="${COMMAND_ARGS[1]:-}"

	# Dispatch to appropriate command handler
	case "${command}" in
		install)
			cmd_install "${version}"
			;;
		deploy)
			cmd_deploy "${version}"
			;;
		rollback)
			cmd_rollback "${version}"
			;;
		uninstall)
			cmd_uninstall "${version}"
			;;
		list)
			cmd_list
			;;
		start)
			cmd_start
			;;
		stop)
			cmd_stop
			;;
		restart)
			restart_service
			;;
		status)
			cmd_status
			;;
		help)
			cmd_usage
			;;
		*)
			log_error "Unknown command: '${command}'. Use '${0} help' for usage."
			;;
	esac
}

# Entry point
main "$@"

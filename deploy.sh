#!/bin/bash
#
# Manages installation, deployment, rollback, and database operations for Senra Server
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
DB_BACKUP_DIR="${DATA_DIR}/backups"
LOG_FILE="/var/log/${SERVICE_NAME}/deploy.log"
LOG_DIR="/var/log/${SERVICE_NAME}"

#######################################
# Parse command line arguments
#######################################
parse_arguments() {
	while [[ ${#} -gt 0 ]]; do
		case "${1}" in
		--repo=*)
			GITHUB_REPO="${1#*=}"
			if [[ -z "${GITHUB_REPO}" ]]; then
				echo "Error: --repo requires a non-empty value" >&2
				exit 1
			fi
			;;
		--binary=*)
			BINARY_NAME="${1#*=}"
			if [[ -z "${BINARY_NAME}" ]]; then
				echo "Error: --binary requires a non-empty value" >&2
				exit 1
			fi
			;;
		--service=*)
			SERVICE_NAME="${1#*=}"
			if [[ -z "${SERVICE_NAME}" ]]; then
				echo "Error: --service requires a non-empty value" >&2
				exit 1
			fi
			;;
		--app-dir=*)
			APP_BASE_DIR="${1#*=}"
			if [[ -z "${APP_BASE_DIR}" ]]; then
				echo "Error: --app-dir requires a non-empty value" >&2
				exit 1
			fi
			;;
		--data-dir=*)
			DATA_DIR="${1#*=}"
			if [[ -z "${DATA_DIR}" ]]; then
				echo "Error: --data-dir requires a non-empty value" >&2
				exit 1
			fi
			;;
		--user=*)
			SERVICE_USER="${1#*=}"
			if [[ -z "${SERVICE_USER}" ]]; then
				echo "Error: --user requires a non-empty value" >&2
				exit 1
			fi
			;;
		--group=*)
			SERVICE_GROUP="${1#*=}"
			if [[ -z "${SERVICE_GROUP}" ]]; then
				echo "Error: --group requires a non-empty value" >&2
				exit 1
			fi
			;;
		--)
			shift
			break
			;;
		install | deploy | rollback | list-releases | db:backup | db:list | db:restore | help | status) break ;;
		*)
			echo "Error: Unknown option '${1}'" >&2
			echo "Use '${0} help' for usage information." >&2
			exit 1
			;;
		esac
		shift
	done

	# Recalculate dependent paths after parsing
	RELEASES_DIR="${APP_BASE_DIR}/releases"
	CURRENT_SYMLINK="${APP_BASE_DIR}/current"
	DB_FILE="${DATA_DIR}/shaderlab.db"
	DB_BACKUP_DIR="${DATA_DIR}/backups"
	LOG_FILE="/var/log/${SERVICE_NAME}/deploy.log"
	LOG_DIR="/var/log/${SERVICE_NAME}"
}

parse_arguments "${@}"

#######################################
# Utility Functions
#######################################

#######################################
# Logs a message with timestamp and level
# Arguments:
#   level: Log level (INFO, WARN, ERROR)
#   message: Log message
#######################################
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

#######################################
# Verifies script is running as root user
#######################################
check_root() {
	if [[ "$(id -u)" -ne 0 ]]; then
		log_error "This script requires root privileges. Use 'sudo ${0} <command>'"
	fi
}

#######################################
# Prompts user for confirmation
# Arguments:
#   prompt: Confirmation prompt text
# Returns:
#   0 if user confirms (y/Y), 1 otherwise
#######################################
confirm() {
	local -r prompt="${1}"
	local choice
	read -r -p "${prompt} (y/N): " choice
	case "${choice}" in
	[Yy] | [Yy][Ee][Ss]) return 0 ;;
	*) return 1 ;;
	esac
}

#######################################
# Ensures directory exists with proper permissions
# Arguments:
#   directory: Directory path
#   owner: Owner (optional)
#   permissions: Permissions (default: 755)
#######################################
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

#######################################
# Checks and installs required system dependencies
# Installs missing packages in batch for efficiency
#######################################
check_dependencies() {
	log_info "Checking system dependencies..."
	local -a missing_deps=()
	local -a package_map=()

	# Check each required dependency
	local -r required_commands=(curl jq setcap sha256sum sqlite3 unzip)
	local cmd
	for cmd in "${required_commands[@]}"; do
		if ! command -v "${cmd}" >/dev/null 2>&1; then
			missing_deps+=("${cmd}")
			# Map command to package name
			case "${cmd}" in
			sha256sum) package_map+=("coreutils") ;;
			setcap) package_map+=("libcap2-bin") ;;
			*) package_map+=("${cmd}") ;;
			esac
		fi
	done

	# Install all missing dependencies at once
	if [[ ${#missing_deps[@]} -gt 0 ]]; then
		log_info "Installing missing dependencies: ${missing_deps[*]} -> packages: ${package_map[*]}"
		if ! apt-get update >/dev/null; then
			log_error "Failed to update package lists"
		fi
		# shellcheck disable=SC2048,SC2086
		if ! apt-get install -y ${package_map[*]}; then
			log_error "Failed to install packages: ${package_map[*]}"
		fi

		# Verify installation
		for cmd in "${missing_deps[@]}"; do
			if ! command -v "${cmd}" >/dev/null 2>&1; then
				log_error "Failed to install dependency: ${cmd}"
			fi
		done
		log_info "All dependencies installed successfully"
	fi
}

#######################################
# Creates service user and group if they don't exist
# Handles cases where user and group have same name
#######################################
create_service_user() {
	# Handle case where user and group have the same name
	if [[ "${SERVICE_USER}" == "${SERVICE_GROUP}" ]]; then
		# Create user with primary group of the same name (system default)
		if ! id "${SERVICE_USER}" >/dev/null 2>&1; then
			log_info "Creating service user and group: ${SERVICE_USER}"
			if ! useradd --system --shell /bin/false --home-dir "${APP_BASE_DIR}" --create-home "${SERVICE_USER}"; then
				log_error "Failed to create service user: ${SERVICE_USER}"
			fi
		else
			log_info "Service user ${SERVICE_USER} already exists"
		fi
	else
		# Create group first, then user
		if ! getent group "${SERVICE_GROUP}" >/dev/null 2>&1; then
			log_info "Creating service group: ${SERVICE_GROUP}"
			if ! groupadd --system "${SERVICE_GROUP}"; then
				log_error "Failed to create service group: ${SERVICE_GROUP}"
			fi
		fi

		if ! id "${SERVICE_USER}" >/dev/null 2>&1; then
			log_info "Creating service user: ${SERVICE_USER}"
			if ! useradd --system --shell /bin/false --home-dir "${APP_BASE_DIR}" --create-home --gid "${SERVICE_GROUP}" "${SERVICE_USER}"; then
				log_error "Failed to create service user: ${SERVICE_USER}"
			fi
		else
			# Add existing user to group if not already member
			if ! id -nG "${SERVICE_USER}" | grep -qw "${SERVICE_GROUP}"; then
				log_info "Adding user ${SERVICE_USER} to group ${SERVICE_GROUP}"
				if ! usermod -a -G "${SERVICE_GROUP}" "${SERVICE_USER}"; then
					log_error "Failed to add user ${SERVICE_USER} to group ${SERVICE_GROUP}"
				fi
			fi
		fi
	fi
}

#######################################
# Sets secure file permissions on all application directories
# Must be called after user/group creation
#######################################
set_secure_permissions() {
	log_info "Setting secure permissions..."

	# Ensure user and group exist before setting permissions
	if ! id "${SERVICE_USER}" >/dev/null 2>&1; then
		log_error "Service user ${SERVICE_USER} does not exist. Run create_service_user first."
	fi

	# App directory: owner=service_user, group=service_group, mode=750
	if [[ -d "${APP_BASE_DIR}" ]]; then
		if ! chown -R "${SERVICE_USER}:${SERVICE_GROUP}" "${APP_BASE_DIR}"; then
			log_error "Failed to set ownership on ${APP_BASE_DIR}"
		fi
		if ! chmod 750 "${APP_BASE_DIR}"; then
			log_error "Failed to set permissions on ${APP_BASE_DIR}"
		fi
	fi

	if [[ -d "${RELEASES_DIR}" ]]; then
		if ! chown -R "${SERVICE_USER}:${SERVICE_GROUP}" "${RELEASES_DIR}"; then
			log_error "Failed to set ownership on ${RELEASES_DIR}"
		fi
		if ! chmod 750 "${RELEASES_DIR}"; then
			log_error "Failed to set permissions on ${RELEASES_DIR}"
		fi
	fi

	# Data directory: owner=service_user, group=service_group, mode=750
	if [[ -d "${DATA_DIR}" ]]; then
		if ! chown -R "${SERVICE_USER}:${SERVICE_GROUP}" "${DATA_DIR}"; then
			log_error "Failed to set ownership on ${DATA_DIR}"
		fi
		if ! chmod 750 "${DATA_DIR}"; then
			log_error "Failed to set permissions on ${DATA_DIR}"
		fi
	fi

	if [[ -d "${DB_BACKUP_DIR}" ]]; then
		if ! chown -R "${SERVICE_USER}:${SERVICE_GROUP}" "${DB_BACKUP_DIR}"; then
			log_error "Failed to set ownership on ${DB_BACKUP_DIR}"
		fi
		if ! chmod 750 "${DB_BACKUP_DIR}"; then
			log_error "Failed to set permissions on ${DB_BACKUP_DIR}"
		fi
	fi

	# Database file: mode=640
	if [[ -f "${DB_FILE}" ]]; then
		if ! chown "${SERVICE_USER}:${SERVICE_GROUP}" "${DB_FILE}"; then
			log_error "Failed to set ownership on ${DB_FILE}"
		fi
		if ! chmod 640 "${DB_FILE}"; then
			log_error "Failed to set permissions on ${DB_FILE}"
		fi
	fi

	# Log directory
	if [[ -d "${LOG_DIR}" ]]; then
		if ! chown -R "${SERVICE_USER}:${SERVICE_GROUP}" "${LOG_DIR}"; then
			log_error "Failed to set ownership on ${LOG_DIR}"
		fi
		if ! chmod 750 "${LOG_DIR}"; then
			log_error "Failed to set permissions on ${LOG_DIR}"
		fi
	fi
}

#######################################
# Performs application health check with configurable timeout
# Arguments:
#   endpoint: Health check endpoint (default: HEALTH_CHECK_ENDPOINT)
#   timeout: Timeout in seconds (default: HEALTH_CHECK_TIMEOUT)
#   port: Port number (default: 80)
# Returns:
#   0 if healthy, 1 if unhealthy
#######################################
health_check() {
	local -r endpoint="${1:-${HEALTH_CHECK_ENDPOINT}}"
	local -r timeout="${2:-${HEALTH_CHECK_TIMEOUT}}"
	local -r port="${3:-80}"

	log_info "Performing health check (timeout: ${timeout}s)..."

	local i
	for ((i = 1; i <= timeout; i++)); do
		# Try health check
		if curl -f -s --max-time 5 "http://localhost:${port}${endpoint}" >/dev/null 2>&1; then
			log_info "Health check passed after ${i}s"
			return 0
		fi

		# Check if service is still running
		if ! systemctl is-active --quiet "${SERVICE_NAME}"; then
			log_error "Service stopped during health check"
			return 1
		fi

		if ((i % 10 == 0)); then
			log_info "Health check still running... (${i}/${timeout}s)"
		fi

		sleep 1
	done

	log_error "Health check failed after ${timeout}s"
	return 1
}

#######################################
# Verifies file integrity using SHA256 checksum
# Arguments:
#   file_path: Path to file to verify
#   expected_checksum: Expected SHA256 checksum
# Returns:
#   0 if checksum matches, 1 otherwise
#######################################
verify_file_integrity() {
	local -r file_path="${1}"
	local expected_checksum="${2}"

	if [[ ! -f "${file_path}" ]]; then
		log_error "File does not exist: ${file_path}"
	fi

	if [[ -z "${expected_checksum}" ]]; then
		log_warn "No checksum provided for verification"
		return 0
	fi

	log_info "Verifying file integrity..."
	local actual_checksum
	if ! actual_checksum=$(sha256sum "${file_path}" | cut -d' ' -f1); then
		log_error "Failed to calculate checksum for ${file_path}"
	fi

	# Clean up expected checksum (remove any extra whitespace or filename)
	expected_checksum=$(echo "${expected_checksum}" | tr -d '[:space:]' | cut -d' ' -f1)

	if [[ "${actual_checksum}" != "${expected_checksum}" ]]; then
		log_error "File integrity check failed. Expected: ${expected_checksum}, Actual: ${actual_checksum}"
	fi

	log_info "File integrity verified successfully"
	return 0
}

#######################################
# Downloads file with retry mechanism
# Arguments:
#   url: URL to download
#   output: Output file path
#   max_retries: Maximum retry attempts (default: 3)
#   retry_delay: Delay between retries (default: 5s)
# Returns:
#   0 if successful, 1 if all retries failed
#######################################
download_with_retry() {
	local -r url="${1}"
	local -r output="${2}"
	local -r max_retries="${3:-3}"
	local -r retry_delay="${4:-5}"

	local i
	for ((i = 1; i <= max_retries; i++)); do
		log_info "Download attempt ${i}/${max_retries}: ${url}"

		if curl -L --fail --retry 2 --retry-delay 2 --max-time 300 "${url}" -o "${output}"; then
			log_info "Download successful"
			return 0
		fi

		if ((i < max_retries)); then
			log_warn "Download failed, retrying in ${retry_delay}s..."
			sleep "${retry_delay}"
		fi
	done

	log_error "Download failed after ${max_retries} attempts"
	return 1
}

#######################################
# Service Management Functions
#######################################

#######################################
# Restarts the systemd service with health check validation
# Returns:
#   0 if service restarted successfully and passes health check
#   1 if restart or health check failed
#######################################
restart_service() {
	log_info "Restarting service: ${SERVICE_NAME}"

	# Check if service exists
	if ! systemctl list-unit-files --type=service | grep -q "^${SERVICE_NAME}.service"; then
		log_error "Service ${SERVICE_NAME} does not exist"
	fi

	# Restart the service
	if ! systemctl restart "${SERVICE_NAME}"; then
		log_error "Failed to restart service: ${SERVICE_NAME}"
	fi

	# Give service time to initialize before checking
	log_info "Waiting for service to initialize..."
	sleep 3

	# Wait for service to start and verify it's running
	local -r max_wait=30
	local i
	for ((i = 1; i <= max_wait; i++)); do
		if systemctl is-active --quiet "${SERVICE_NAME}"; then
			log_info "Service is active, waiting a bit more for full startup..."
			sleep 2 # Give service time to fully start

			# Perform health check
			if health_check; then
				log_info "Service restarted and health check passed"
				return 0
			else
				log_error "Service started but health check failed"
				systemctl status "${SERVICE_NAME}" --no-pager >&2
				return 1
			fi
		fi

		if ((i % 5 == 0)); then
			log_info "Still waiting for service to start... (${i}/${max_wait}s)"
		fi

		sleep 1
	done

	log_error "Service failed to start within ${max_wait}s"
	systemctl status "${SERVICE_NAME}" --no-pager >&2
	return 1
}

#######################################
# Checks if the systemd service is currently running
# Returns:
#   0 if service is active, 1 otherwise
#######################################
is_service_running() {
	systemctl is-active --quiet "${SERVICE_NAME}" 2>/dev/null
}

#######################################
# Fetches latest release information from GitHub API
# Outputs download URL and optional SHA256 digest to stdout
# Returns:
#   0 if successful, 1 if failed
#######################################
get_latest_release() {
	log_info "Fetching latest release information..."

	local release_info
	if ! release_info=$(curl -s "https://api.github.com/repos/${GITHUB_REPO}/releases/latest"); then
		log_error "Failed to fetch release information from GitHub API"
	fi

	if [[ -z "${release_info}" || "${release_info}" == "null" ]]; then
		log_error "Invalid release information received"
	fi

	local asset_info
	if ! asset_info=$(echo "${release_info}" | jq -r ".assets[] | select(.name | contains(\"${BINARY_NAME}.zip\"))"); then
		log_error "Failed to parse release information"
	fi

	if [[ -z "${asset_info}" || "${asset_info}" == "null" ]]; then
		log_error "Binary asset not found in release"
	fi

	local download_url
	if ! download_url=$(echo "${asset_info}" | jq -r ".browser_download_url"); then
		log_error "Failed to extract download URL"
	fi

	# Get GitHub's calculated SHA256 digest
	local sha256_digest
	sha256_digest=$(echo "${asset_info}" | jq -r ".digest // null")
	if [[ "${sha256_digest}" != "null" && -n "${sha256_digest}" ]]; then
		sha256_digest=$(echo "${sha256_digest}" | sed 's/^sha256://')
	else
		sha256_digest=""
	fi

	echo "${download_url}"
	if [[ -n "${sha256_digest}" ]]; then
		echo "${sha256_digest}"
	fi
}

#######################################
# Command Implementations
#######################################

#######################################
# Installs and configures the application environment
# Creates users, directories, and systemd service
#######################################
cmd_install() {
	check_root
	log_info "Initializing environment..."

	# Create log directory first
	ensure_directory "${LOG_DIR}" "" "755"

	check_dependencies
	create_service_user

	if [ -f "${DB_FILE}" ]; then
		sudo -u "${SERVICE_USER}" sqlite3 "${DB_FILE}" "VACUUM;"
	fi

	# Create directories with proper ownership
	ensure_directory "${APP_BASE_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"
	ensure_directory "${RELEASES_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"
	ensure_directory "${DATA_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"
	ensure_directory "${DB_BACKUP_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"

	# Set permissions after all directories are created and user exists
	set_secure_permissions

	# Create systemd service with security hardening
	cat <<EOF >"/etc/systemd/system/${SERVICE_NAME}.service"
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
ProtectSystem=full
ProtectHome=true
ReadWritePaths=${DATA_DIR} ${LOG_DIR}
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Environment variables
Environment="HOST=0.0.0.0"
Environment="PORT=80"
Environment="DATABASE_URL=sqlite:file:${DB_FILE}"
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
EOF

	systemctl daemon-reload
	log_info "Environment initialized successfully. Run 'deploy' to install the application."
}

cmd_deploy() {
	check_root
	log_info "Starting deployment of latest version..."

	# Ensure releases directory exists
	ensure_directory "${RELEASES_DIR}" "${SERVICE_USER}:${SERVICE_GROUP}" "750"

	# Get release information
	local -a release_data
	mapfile -t release_data < <(get_latest_release)
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
	local temp_dir_created=true

	# Ensure cleanup on exit, but handle the mv case
	cleanup_temp() {
		if [[ "${temp_dir_created}" == "true" && -d "${temp_dir}" ]]; then
			log_info "Cleaning up temporary directory"
			rm -rf "${temp_dir}"
		fi
	}
	trap cleanup_temp EXIT

	# Download release
	download_with_retry "${asset_url}" "${temp_dir}/release.zip"

	# Verify checksum if available from GitHub API
	if [[ -n "${sha256_digest}" ]]; then
		verify_file_integrity "${temp_dir}/release.zip" "${sha256_digest}"
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

	# Mark temp directory as moved to prevent cleanup
	temp_dir_created=false

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
		cmd_cleanup_releases
	else
		log_error "Deployment failed during service restart"
	fi
}

cmd_rollback() {
	check_root
	log_info "Starting rollback to previous version..."

	# Ensure releases directory exists
	if [[ ! -d "${RELEASES_DIR}" ]]; then
		log_error "Releases directory ${RELEASES_DIR} does not exist"
	fi

	# Get sorted list of valid releases
	local -a releases=()
	while IFS= read -r -d $'\0'; do
		local base
		base=$(basename "${REPLY}")
		if [[ "${base}" =~ ^[0-9]{14,}$ ]]; then
			releases+=("${REPLY}")
		fi
	done < <(find "${RELEASES_DIR}" -maxdepth 1 -mindepth 1 -type d -print0 2>/dev/null | sort -z)

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

cmd_db_backup() {
	check_root

	# Ensure backup directory exists
	ensure_directory "$DB_BACKUP_DIR" "$SERVICE_USER:$SERVICE_GROUP" "750"

	[[ ! -f "$DB_FILE" ]] && {
		log_warn "Database file not found, skipping backup"
		return 0
	}

	local backup_file="$DB_BACKUP_DIR/shaderlab.db-$(date +%Y%m%d-%H%M%S).bak"
	log_info "Creating database backup: $(basename "$backup_file")"

	# Check if service is running
	local was_running=false
	if is_service_running; then
		was_running=true
		log_info "Stopping service for consistent backup..."
		systemctl stop "$SERVICE_NAME"
	fi

	# Create backup
	cp "$DB_FILE" "$backup_file"

	# Set proper ownership
	chown "$SERVICE_USER:$SERVICE_GROUP" "$backup_file"
	chmod 640 "$backup_file"

	# Restart service only if it was running
	if [[ "$was_running" == "true" ]]; then
		log_info "Restarting service..."
		systemctl start "$SERVICE_NAME"
	fi

	log_info "Database backup completed successfully"
}

cmd_db_restore() {
	check_root
	local backup_file="$DB_BACKUP_DIR/${1:?Backup filename required}"
	[[ ! -f "$backup_file" ]] && log_error "Backup file not found: $backup_file"

	confirm "Restore database from $(basename "$backup_file")? This will overwrite the current database!" || exit 0

	log_info "Restoring database from: $(basename "$backup_file")"

	# Check if service is running
	local was_running=false
	if is_service_running; then
		was_running=true
		log_info "Stopping service for database restore..."
		systemctl stop "$SERVICE_NAME"
	fi

	# Restore database
	cp -f "$backup_file" "$DB_FILE"

	# Set proper ownership
	chown "$SERVICE_USER:$SERVICE_GROUP" "$DB_FILE"
	chmod 640 "$DB_FILE"

	# Restart service only if it was running
	if [[ "$was_running" == "true" ]]; then
		log_info "Restarting service..."
		systemctl start "$SERVICE_NAME"
	fi

	log_info "Database restore completed successfully"
}

cmd_cleanup_releases() {
	log_info "Cleaning up old releases..."

	# Check if releases directory exists
	if [[ ! -d "${RELEASES_DIR}" ]]; then
		log_warn "Releases directory ${RELEASES_DIR} does not exist"
		return 0
	fi

	# Keep only the latest 3 valid timestamp-named release directories
	local -a releases=()
	while IFS= read -r -d $'\0'; do
		local base
		base=$(basename "${REPLY}")
		if [[ "${base}" =~ ^[0-9]{14,}$ ]]; then
			releases+=("${REPLY}")
		fi
	done < <(find "${RELEASES_DIR}" -maxdepth 1 -mindepth 1 -type d -print0 2>/dev/null | sort -z)

	local -r n=${#releases[@]}
	local cleaned=0

	local i
	for ((i = 0; i < n - 3; i++)); do
		log_info "Removing old release: $(basename "${releases[i]}")"
		if rm -rf "${releases[i]}"; then
			((cleaned++))
		else
			log_warn "Failed to remove release: $(basename "${releases[i]}")"
		fi
	done

	log_info "Cleanup completed: removed ${cleaned} old releases"
}

cmd_list_releases() {
	# Check if releases directory exists
	if [[ ! -d "${RELEASES_DIR}" ]]; then
		echo "No releases directory found at ${RELEASES_DIR}"
		return 0
	fi

	echo "Available releases:"

	local -a releases=()
	while IFS= read -r -d $'\0'; do
		local base
		base=$(basename "${REPLY}")
		if [[ "${base}" =~ ^[0-9]{14,}$ ]]; then
			releases+=("${base}")
		fi
	done < <(find "${RELEASES_DIR}" -maxdepth 1 -mindepth 1 -type d -print0 2>/dev/null)

	if [[ ${#releases[@]} -eq 0 ]]; then
		echo "  No releases found"
		return 0
	fi

	# Sort releases in descending order
	IFS=$'\n' releases=($(sort -r <<<"${releases[*]}"))
	IFS=$'\n\t'

	local release
	for release in "${releases[@]}"; do
		if [[ -L "${CURRENT_SYMLINK}" ]] && [[ "$(readlink "${CURRENT_SYMLINK}")" == "${RELEASES_DIR}/${release}" ]]; then
			echo "  ${release} (current)"
		else
			echo "  ${release}"
		fi
	done
}

cmd_status() {
	echo "=== Service Status ==="
	if systemctl list-unit-files --type=service | grep -q "^${SERVICE_NAME}.service"; then
		systemctl status "${SERVICE_NAME}" --no-pager || true
	else
		echo "Service ${SERVICE_NAME} is not installed"
	fi

	echo -e "\n=== Current Release ==="
	if [[ -L "${CURRENT_SYMLINK}" ]]; then
		local current_target
		current_target=$(readlink "${CURRENT_SYMLINK}")
		if [[ -d "${current_target}" ]]; then
			echo "Current: $(basename "${current_target}")"
		else
			echo "Current symlink points to non-existent directory: ${current_target}"
		fi
	else
		echo "No current release symlink found"
	fi

	echo -e "\n=== Health Check ==="
	if systemctl list-unit-files --type=service | grep -q "^${SERVICE_NAME}.service" && is_service_running; then
		if health_check "" 5; then
			echo "Service is healthy"
		else
			echo "Service health check failed"
		fi
	else
		echo "Service is not running"
	fi
}

cmd_usage() {
	cat <<EOF
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
  install                 Initialize environment and install service
  deploy                  Deploy latest version with health checks
  rollback                Rollback to previous version
  status                  Show service status and health
  list-releases           List available releases

Database Management:
  db:backup               Create database backup
  db:list                 List available backups
  db:restore <filename>   Restore from backup

  help                    Show this help

Examples:
  # Standard installation
  sudo ${0} install
  sudo ${0} deploy
  
  # Custom configuration
  sudo ${0} --user=myuser --data-dir=/custom/data install
  
  # Check service health
  sudo ${0} status
EOF
}

# =========================
# Main Entry
# =========================
main() {
	# Ensure log directory exists if we're root
	if [[ "$(id -u)" -eq 0 ]] && [[ ! -d "${LOG_DIR}" ]]; then
		if ! mkdir -p "${LOG_DIR}"; then
			echo "Warning: Failed to create log directory ${LOG_DIR}" >&2
		fi
	fi

	case "${1:-help}" in
	install) cmd_install ;;
	deploy) cmd_deploy ;;
	rollback) cmd_rollback ;;
	status) cmd_status ;;
	list-releases) cmd_list_releases ;;
	db:backup) cmd_db_backup ;;
	db:list)
		if [[ -d "${DB_BACKUP_DIR}" ]]; then
			find "${DB_BACKUP_DIR}" -type f -name "*.bak" -printf "%f\n" 2>/dev/null | sort -r || echo "No backups found"
		else
			echo "Backup directory ${DB_BACKUP_DIR} does not exist"
		fi
		;;
	db:restore)
		shift
		cmd_db_restore "${1:-}"
		;;
	help) cmd_usage ;;
	*)
		cmd_usage
		exit 1
		;;
	esac
}

main "${@}"

#!/bin/bash
set -euo pipefail

# CivitForge Security Audit Script
# This script runs various security checks and tools

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_tool_installed() {
    local tool=$1
    if ! command -v "$tool" &> /dev/null; then
        log_error "$tool is not installed. Please install it first."
        return 1
    fi
    return 0
}

# Create results directory
mkdir -p security-audit-results
RESULTS_DIR="security-audit-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/audit_results_$TIMESTAMP.txt"

# Initialize results file
echo "CivitForge Security Audit Results" > "$RESULTS_FILE"
echo "=================================" >> "$RESULTS_FILE"
echo "Timestamp: $(date)" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Function to run a check and record results
run_check() {
    local check_name=$1
    local command=$2
    local required=$3  # 1 = required, 0 = optional
    
    log_info "Running check: $check_name"
    echo "Check: $check_name" >> "$RESULTS_FILE"
    echo "Command: $command" >> "$RESULTS_FILE"
    
    if eval "$command" >> "$RESULTS_FILE" 2>&1; then
        log_success "✓ $check_name passed"
        echo "Result: PASS" >> "$RESULTS_FILE"
    else
        if [ "$required" -eq 1 ]; then
            log_error "✗ $check_name failed (REQUIRED)"
            echo "Result: FAIL (REQUIRED)" >> "$RESULTS_FILE"
        else
            log_warning "⚠ $check_name failed (optional)"
            echo "Result: FAIL (optional)" >> "$RESULTS_FILE"
        fi
    fi
    echo "" >> "$RESULTS_FILE"
}

# Function to check for hardcoded secrets
check_hardcoded_secrets() {
    log_info "Checking for hardcoded secrets..."
    
    # Check for common secret patterns
    local secret_patterns=(
        "password\s*=\s*[\"'][^\"']+[\"']"
        "secret\s*=\s*[\"'][^\"']+[\"']"
        "api_key\s*=\s*[\"'][^\"']+[\"']"
        "token\s*=\s*[\"'][^\"']+[\"']"
        "PRIVATE KEY"
        "BEGIN RSA PRIVATE KEY"
        "BEGIN DSA PRIVATE KEY"
        "BEGIN EC PRIVATE KEY"
        "BEGIN OPENSSH PRIVATE KEY"
    )
    
    local found_secrets=0
    
    for pattern in "${secret_patterns[@]}"; do
        if grep -r -i -E "$pattern" --include="*.rs" --include="*.toml" --include="*.yml" --include="*.yaml" --include="*.json" --include="*.env" --include="*.sh" . 2>/dev/null | grep -v "target/" | grep -v ".git/" | grep -v "CHANGELOG.md" | grep -v "README.md" | grep -v "CONTRIBUTING.md" | grep -v "CODE_OF_CONDUCT.md" | head -20; then
            found_secrets=$((found_secrets + 1))
        fi
    done
    
    if [ $found_secrets -eq 0 ]; then
        log_success "No hardcoded secrets found"
        echo "No hardcoded secrets found" >> "$RESULTS_FILE"
    else
        log_warning "Potential hardcoded secrets found"
        echo "Potential hardcoded secrets found: $found_secrets" >> "$RESULTS_FILE"
    fi
    echo "" >> "$RESULTS_FILE"
}

# Function to check for dependency vulnerabilities
check_dependencies() {
    log_info "Checking for dependency vulnerabilities..."
    
    # Check for Cargo audit
    if check_tool_installed "cargo-audit"; then
        log_info "Running cargo audit..."
        cargo audit >> "$RESULTS_FILE" 2>&1 || true
    else
        log_warning "cargo-audit not installed, skipping cargo audit"
        echo "cargo-audit not installed" >> "$RESULTS_FILE"
    fi
    
    # Check for outdated dependencies
    if check_tool_installed "cargo-outdated"; then
        log_info "Checking for outdated dependencies..."
        cargo outdated >> "$RESULTS_FILE" 2>&1 || true
    fi
    
    echo "" >> "$RESULTS_FILE"
}

# Function to check for container security issues
check_container_security() {
    log_info "Checking container security..."
    
    # Check if Docker is available
    if check_tool_installed "docker"; then
        log_info "Checking Docker images..."
        
        # Check for trivy
        if check_tool_installed "trivy"; then
            log_info "Running trivy image scan..."
            
            # Scan CivitForge images if they exist
            local images=("civitforge:latest" "civitforge-core:latest" "civitforge-brain:latest")
            for image in "${images[@]}"; do
                if docker image inspect "$image" &> /dev/null; then
                    log_info "Scanning image: $image"
                    trivy image --severity HIGH,CRITICAL "$image" >> "$RESULTS_FILE" 2>&1 || true
                fi
            done
        else
            log_warning "trivy not installed, skipping image scanning"
            echo "trivy not installed" >> "$RESULTS_FILE"
        fi
        
        # Check Docker Compose configuration
        if [ -f "docker-compose.yml" ]; then
            log_info "Checking Docker Compose configuration..."
            docker compose config --quiet >> "$RESULTS_FILE" 2>&1 || true
        fi
    else
        log_warning "Docker not installed, skipping container security checks"
        echo "Docker not installed" >> "$RESULTS_FILE"
    fi
    echo "" >> "$RESULTS_FILE"
}

# Function to check for security headers
check_security_headers() {
    log_info "Checking security headers..."
    
    # This would typically require a running server
    # For now, we'll just check if the code includes security headers
    
    local security_headers=(
        "Content-Security-Policy"
        "X-Content-Type-Options"
        "X-Frame-Options"
        "Strict-Transport-Security"
        "X-XSS-Protection"
        "Referrer-Policy"
        "Permissions-Policy"
    )
    
    local found_headers=0
    
    for header in "${security_headers[@]}"; do
        if grep -r -i "$header" --include="*.rs" --include="*.toml" --include="*.yml" --include="*.yaml" . 2>/dev/null | grep -v "target/" | grep -v ".git/" | head -5; then
            found_headers=$((found_headers + 1))
        fi
    done
    
    if [ $found_headers -gt 0 ]; then
        log_success "Found $found_headers security headers in code"
        echo "Found $found_headers security headers in code" >> "$RESULTS_FILE"
    else
        log_warning "No security headers found in code"
        echo "No security headers found in code" >> "$RESULTS_FILE"
    fi
    echo "" >> "$RESULTS_FILE"
}

# Function to check for input validation
check_input_validation() {
    log_info "Checking for input validation..."
    
    # Check for common validation patterns
    local validation_patterns=(
        "validate"
        "sanitiz"
        "escape"
        "encode"
        "decode"
        "parse"
        "verify"
        "check"
        "assert"
    )
    
    local found_validations=0
    
    for pattern in "${validation_patterns[@]}"; do
        local count=$(grep -r -i -E "$pattern" --include="*.rs" . 2>/dev/null | grep -v "target/" | grep -v ".git/" | wc -l)
        found_validations=$((found_validations + count))
    done
    
    if [ $found_validations -gt 0 ]; then
        log_success "Found $found_validations input validation related functions"
        echo "Found $found_validations input validation related functions" >> "$RESULTS_FILE"
    else
        log_warning "No input validation functions found"
        echo "No input validation functions found" >> "$RESULTS_FILE"
    fi
    echo "" >> "$RESULTS_FILE"
}

# Function to check for error handling
check_error_handling() {
    log_info "Checking for error handling..."
    
    # Check for error handling patterns
    local error_patterns=(
        "unwrap_or"
        "unwrap_or_else"
        "expect"
        "Result<"
        "Error"
        "panic!"
        "assert!"
    )
    
    local found_error_handling=0
    
    for pattern in "${error_patterns[@]}"; do
        local count=$(grep -r -i -E "$pattern" --include="*.rs" . 2>/dev/null | grep -v "target/" | grep -v ".git/" | wc -l)
        found_error_handling=$((found_error_handling + count))
    done
    
    if [ $found_error_handling -gt 0 ]; then
        log_success "Found $found_error_handling error handling patterns"
        echo "Found $found_error_handling error handling patterns" >> "$RESULTS_FILE"
    else
        log_warning "No error handling patterns found"
        echo "No error handling patterns found" >> "$RESULTS_FILE"
    fi
    echo "" >> "$RESULTS_FILE"
}

# Function to check for logging and monitoring
check_logging() {
    log_info "Checking for logging and monitoring..."
    
    # Check for logging patterns
    local logging_patterns=(
        "log::"
        "tracing::"
        "info!"
        "warn!"
        "error!"
        "debug!"
        "trace!"
        "println!"
        "eprintln!"
    )
    
    local found_logging=0
    
    for pattern in "${logging_patterns[@]}"; do
        local count=$(grep -r -i -E "$pattern" --include="*.rs" . 2>/dev/null | grep -v "target/" | grep -v ".git/" | wc -l)
        found_logging=$((found_logging + count))
    done
    
    if [ $found_logging -gt 0 ]; then
        log_success "Found $found_logging logging statements"
        echo "Found $found_logging logging statements" >> "$RESULTS_FILE"
    else
        log_warning "No logging statements found"
        echo "No logging statements found" >> "$RESULTS_FILE"
    fi
    echo "" >> "$RESULTS_FILE"
}

# Main execution
main() {
    log_info "Starting CivitForge Security Audit"
    echo "Starting CivitForge Security Audit" >> "$RESULTS_FILE"
    echo "" >> "$RESULTS_FILE"
    
    # Run all checks
    check_hardcoded_secrets
    check_dependencies
    check_container_security
    check_security_headers
    check_input_validation
    check_error_handling
    check_logging
    
    # Additional checks from the security audit checklist
    log_info "Running additional security checks..."
    
    # Check for SQL injection prevention (Rust's type system helps here)
    run_check "SQL Injection Prevention" "grep -r 'sqlx::query\|sqlx::query_as\|diesel::' --include='*.rs' . | grep -v 'target/' | grep -v '.git/' | wc -l | grep -v '^0$'" 0
    
    # Check for XSS prevention
    run_check "XSS Prevention" "grep -r 'escape\|encode\|sanitize\|html::' --include='*.rs' . | grep -v 'target/' | grep -v '.git/' | wc -l | grep -v '^0$'" 0
    
    # Check for CSRF protection
    run_check "CSRF Protection" "grep -r 'csrf\|token\|session' --include='*.rs' . | grep -v 'target/' | grep -v '.git/' | wc -l | grep -v '^0$'" 0
    
    # Check for rate limiting
    run_check "Rate Limiting" "grep -r 'rate.*limit\|throttl\|limit.*rate' --include='*.rs' . | grep -v 'target/' | grep -v '.git/' | wc -l | grep -v '^0$'" 0
    
    # Check for secret management
    run_check "Secret Management" "grep -r 'env::var\|dotenv\|config\|secret\|key' --include='*.rs' . | grep -v 'target/' | grep -v '.git/' | wc -l | grep -v '^0$'" 0
    
    # Check for encryption
    run_check "Encryption" "grep -r 'encrypt\|decrypt\|cipher\|hash\|bcrypt\|argon2\|sha256\|sha512' --include='*.rs' . | grep -v 'target/' | grep -v '.git/' | wc -l | grep -v '^0$'" 0
    
    # Check for audit logging
    run_check "Audit Logging" "grep -r 'audit\|log\|trace\|monitor' --include='*.rs' . | grep -v 'target/' | grep -v '.git/' | wc -l | grep -v '^0$'" 0
    
    # Check for compliance-related code
    run_check "Compliance Code" "grep -r 'gdpr\|ccpa\|privacy\|consent\|data.*protection' --include='*.rs' . | grep -v 'target/' | grep -v '.git/' | wc -l | grep -v '^0$'" 0
    
    log_success "Security audit completed"
    echo "" >> "$RESULTS_FILE"
    echo "Security audit completed at $(date)" >> "$RESULTS_FILE"
    
    echo ""
    log_info "Results saved to: $RESULTS_FILE"
    echo ""
    log_info "Summary:"
    echo "--------"
    grep -E "^(Check|Result):" "$RESULTS_FILE" | paste - - | column -t -s $'\t'
}

# Run main function
main "$@"
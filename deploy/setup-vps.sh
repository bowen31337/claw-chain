#!/bin/bash
# ClawChain VPS Setup Script
# One-command deployment for ClawChain Substrate testnet
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/clawinfra/claw-chain/main/deploy/setup-vps.sh | bash
#
# Or download and run:
#   wget https://raw.githubusercontent.com/clawinfra/claw-chain/main/deploy/setup-vps.sh
#   chmod +x setup-vps.sh
#   ./setup-vps.sh

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/clawinfra/claw-chain.git"
REPO_DIR="$HOME/claw-chain"
IMAGE_NAME="localhost/clawchain-node:latest"
QUADLET_DIR="$HOME/.config/containers/systemd"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            echo "amd64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Detect package manager
detect_pkg_manager() {
    if command -v apt-get &> /dev/null; then
        echo "apt"
    elif command -v dnf &> /dev/null; then
        echo "dnf"
    elif command -v yum &> /dev/null; then
        echo "yum"
    else
        log_error "No supported package manager found (apt, dnf, yum)"
        exit 1
    fi
}

# Install Podman
install_podman() {
    log_info "Checking for Podman..."
    
    if command -v podman &> /dev/null; then
        local version=$(podman --version | cut -d' ' -f3)
        log_success "Podman already installed: $version"
        return 0
    fi

    log_info "Installing Podman..."
    local pkg_mgr=$(detect_pkg_manager)

    case $pkg_mgr in
        apt)
            sudo apt-get update
            sudo apt-get install -y podman
            ;;
        dnf)
            sudo dnf install -y podman
            ;;
        yum)
            sudo yum install -y podman
            ;;
    esac

    log_success "Podman installed successfully"
}

# Clone or update repository
setup_repository() {
    if [ -d "$REPO_DIR" ]; then
        log_info "Repository already exists, updating..."
        cd "$REPO_DIR"
        git pull origin main
    else
        log_info "Cloning repository..."
        git clone "$REPO_URL" "$REPO_DIR"
        cd "$REPO_DIR"
    fi
    
    log_success "Repository ready at $REPO_DIR"
}

# Build container image
build_image() {
    log_info "Building ClawChain container image..."
    log_warn "This may take 30-60 minutes on first build (compiling Rust)"
    
    cd "$REPO_DIR"
    
    # Build with Podman
    podman build \
        -t "$IMAGE_NAME" \
        -f deploy/Containerfile \
        .
    
    log_success "Container image built: $IMAGE_NAME"
}

# Alternatively, pull pre-built image (if available)
pull_image() {
    log_info "Attempting to pull pre-built image..."
    
    # TODO: Update when image is published to a registry
    # podman pull ghcr.io/clawinfra/clawchain-node:latest
    # podman tag ghcr.io/clawinfra/clawchain-node:latest "$IMAGE_NAME"
    
    log_warn "Pre-built images not yet available, building locally..."
    build_image
}

# Setup Quadlet systemd files
setup_quadlet() {
    log_info "Setting up Podman Quadlet systemd files..."
    
    # Create Quadlet directory
    mkdir -p "$QUADLET_DIR"
    
    # Copy Quadlet files
    cp "$REPO_DIR/deploy/quadlet/clawchain-node.container" "$QUADLET_DIR/"
    cp "$REPO_DIR/deploy/quadlet/clawchain-data.volume" "$QUADLET_DIR/"
    
    # Optional: Copy dev mode
    # cp "$REPO_DIR/deploy/quadlet/clawchain-node-dev.container" "$QUADLET_DIR/"
    
    log_success "Quadlet files copied to $QUADLET_DIR"
}

# Enable and start services
start_services() {
    log_info "Reloading systemd daemon..."
    systemctl --user daemon-reload
    
    log_info "Enabling ClawChain node service..."
    systemctl --user enable clawchain-node.service
    
    log_info "Starting ClawChain node..."
    systemctl --user start clawchain-node.service
    
    # Enable lingering to keep services running after logout
    loginctl enable-linger "$USER"
    
    log_success "ClawChain node service started"
}

# Display status
show_status() {
    echo ""
    log_info "==================================================================="
    log_success "ClawChain Deployment Complete!"
    log_info "==================================================================="
    echo ""
    
    # Service status
    echo -e "${BLUE}Service Status:${NC}"
    systemctl --user status clawchain-node.service --no-pager || true
    echo ""
    
    # Container status
    echo -e "${BLUE}Container Status:${NC}"
    podman ps --filter name=clawchain-node
    echo ""
    
    # Network info
    local ip=$(hostname -I | awk '{print $1}')
    echo -e "${BLUE}Network Access:${NC}"
    echo "  RPC WebSocket:    ws://$ip:9944"
    echo "  RPC HTTP:         http://$ip:9944"
    echo "  Prometheus:       http://$ip:9615/metrics"
    echo "  P2P:              $ip:30333"
    echo ""
    
    # Useful commands
    echo -e "${BLUE}Useful Commands:${NC}"
    echo "  View logs:        journalctl --user -u clawchain-node -f"
    echo "  Stop node:        systemctl --user stop clawchain-node"
    echo "  Start node:       systemctl --user start clawchain-node"
    echo "  Restart node:     systemctl --user restart clawchain-node"
    echo "  Container shell:  podman exec -it clawchain-node sh"
    echo ""
    
    # Polkadot.js Apps
    echo -e "${BLUE}Connect to Polkadot.js Apps:${NC}"
    echo "  https://polkadot.js.org/apps/?rpc=ws://$ip:9944"
    echo ""
    
    log_info "==================================================================="
}

# Main installation flow
main() {
    log_info "Starting ClawChain VPS setup..."
    log_info "Architecture: $(detect_arch)"
    log_info "Package manager: $(detect_pkg_manager)"
    echo ""
    
    # Install dependencies
    install_podman
    
    # Setup repository
    setup_repository
    
    # Build or pull image
    if [ "${CLAWCHAIN_PULL_IMAGE:-0}" = "1" ]; then
        pull_image
    else
        build_image
    fi
    
    # Setup Quadlet
    setup_quadlet
    
    # Start services
    start_services
    
    # Wait for node to initialize
    log_info "Waiting for node to initialize (60 seconds)..."
    sleep 60
    
    # Show status
    show_status
}

# Run main function
main "$@"

# ClawChain Deployment

Quick reference for deploying ClawChain Substrate testnet using Podman + Quadlet.

## 🚀 Quick Start

### One-Command VPS Setup

```bash
curl -fsSL https://raw.githubusercontent.com/clawinfra/claw-chain/main/deploy/setup-vps.sh | bash
```

Or download and customize:

```bash
wget https://raw.githubusercontent.com/clawinfra/claw-chain/main/deploy/setup-vps.sh
chmod +x setup-vps.sh
./setup-vps.sh
```

### Manual Setup

1. **Install Podman**
   ```bash
   # Debian/Ubuntu
   sudo apt-get install podman

   # Fedora/RHEL
   sudo dnf install podman
   ```

2. **Build Container Image**
   ```bash
   cd /path/to/claw-chain
   podman build -t localhost/clawchain-node:latest -f deploy/Containerfile .
   ```

3. **Install Quadlet Files**
   ```bash
   mkdir -p ~/.config/containers/systemd
   cp deploy/quadlet/*.{container,volume} ~/.config/containers/systemd/
   systemctl --user daemon-reload
   ```

4. **Start Service**
   ```bash
   systemctl --user start clawchain-node
   systemctl --user enable clawchain-node
   loginctl enable-linger $USER
   ```

## 📁 File Structure

```
deploy/
├── Containerfile                  # Multi-stage Podman build
├── setup-vps.sh                   # Automated deployment script
├── quadlet/
│   ├── clawchain-node.container   # Production validator
│   ├── clawchain-node-dev.container  # Dev mode
│   ├── clawchain-data.volume      # Persistent data
│   ├── clawchain-proxy.container  # Nginx reverse proxy
│   ├── prometheus.container       # Monitoring (optional)
│   └── prometheus-data.volume     # Metrics storage
├── nginx/
│   └── nginx.conf                 # Reverse proxy config
└── monitoring/
    └── prometheus.yml             # Metrics scraping config
```

## 🔧 Configuration

### Node Modes

**Validator (Production):** `clawchain-node.container`
- Persistent data volume
- Safe RPC methods only
- Prometheus metrics enabled

**Development:** `clawchain-node-dev.container`
- Ephemeral storage (--tmp)
- Unsafe RPC methods
- Debug logging

### Ports

| Port  | Service           | Protocol |
|-------|-------------------|----------|
| 9944  | RPC (WebSocket)   | ws/http  |
| 9615  | Prometheus        | http     |
| 30333 | P2P Networking    | tcp/udp  |

### Environment Variables

Set in `.container` files:

```ini
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1
```

## 📊 Monitoring

### Local Prometheus

```bash
cp deploy/quadlet/prometheus.container ~/.config/containers/systemd/
cp deploy/quadlet/prometheus-data.volume ~/.config/containers/systemd/
systemctl --user daemon-reload
systemctl --user start prometheus
```

Access: http://localhost:9090

### Grafana Cloud (Free Tier)

1. Sign up at https://grafana.com/products/cloud/
2. Get Prometheus remote_write credentials
3. Update `deploy/monitoring/prometheus.yml` with credentials
4. Import Substrate node dashboard

## 🔐 Security

- Node runs as non-root user (UID 1000)
- Safe RPC methods in production
- Rate limiting via Nginx proxy
- Health checks enabled

## 🛠️ Useful Commands

```bash
# View logs
journalctl --user -u clawchain-node -f

# Container shell
podman exec -it clawchain-node sh

# Check node health
curl http://localhost:9944/health

# View metrics
curl http://localhost:9615/metrics

# Restart service
systemctl --user restart clawchain-node

# Stop service
systemctl --user stop clawchain-node

# Remove data (⚠️ destructive)
systemctl --user stop clawchain-node
podman volume rm clawchain-data
```

## 🌐 Connect Polkadot.js Apps

https://polkadot.js.org/apps/?rpc=ws://YOUR_IP:9944

## 🐞 Troubleshooting

**Service won't start:**
```bash
systemctl --user status clawchain-node
journalctl --user -u clawchain-node -n 50
```

**Container build fails:**
```bash
# Check Rust version
podman run --rm docker.io/rust:1.93-bookworm cargo --version

# Build with verbose output
podman build --log-level=debug -t localhost/clawchain-node:latest -f deploy/Containerfile .
```

**Port conflicts:**
```bash
# Check what's using ports
ss -tulpn | grep -E ':(9944|9615|30333)'
```

## 📚 Documentation

- Full deployment guide: [docs/deployment.md](../docs/deployment.md)
- Architecture: See ASCII diagram in deployment.md
- Scaling plans: Phase 1/2/3 in deployment.md

## 🆘 Support

- GitHub Issues: https://github.com/clawinfra/claw-chain/issues
- Discord: [Coming soon]

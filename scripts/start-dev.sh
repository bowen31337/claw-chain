#!/bin/bash
# Start ClawChain dev testnet
# Usage: ./start-dev.sh [--persistent]

BINARY="/home/bowen/claw-chain/target/release/clawchain-node"
LOGFILE="/tmp/clawchain-node.log"
PIDFILE="/tmp/clawchain-node.pid"

# Check if already running
if [ -f "$PIDFILE" ] && kill -0 $(cat "$PIDFILE") 2>/dev/null; then
    echo "ClawChain node already running (PID: $(cat $PIDFILE))"
    exit 0
fi

# Default: --tmp (ephemeral, fresh state each restart)
# With --persistent: uses /home/bowen/.clawchain/dev
STORAGE_FLAG="--tmp"
DATA_DIR="/home/bowen/.clawchain/dev"
if [ "$1" = "--persistent" ]; then
    mkdir -p "$DATA_DIR"
    STORAGE_FLAG="--base-path $DATA_DIR"
fi

nohup "$BINARY" \
    --dev \
    $STORAGE_FLAG \
    --rpc-cors all \
    --rpc-methods unsafe \
    --rpc-external \
    --rpc-port 9944 \
    > "$LOGFILE" 2>&1 &

echo $! > "$PIDFILE"
echo "ClawChain dev node started (PID: $!)"
echo "RPC: ws://localhost:9944"
echo "Log: $LOGFILE"
echo ""
echo "Connect via Polkadot.js Apps:"
echo "  https://polkadot.js.org/apps/?rpc=ws://localhost:9944"

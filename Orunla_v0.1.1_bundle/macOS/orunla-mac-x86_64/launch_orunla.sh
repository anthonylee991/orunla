#!/bin/bash
# ORUNLA - Agent Memory System
# Launcher for Mac/Linux

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ORUNLA - Agent Memory System"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if orunla_cli exists
if [ ! -f "./orunla_cli" ]; then
    echo "❌ Error: orunla_cli not found"
    echo ""
    echo "Please build the CLI tool first:"
    echo "  cargo build --release --bin orunla_cli"
    echo "  cp target/release/orunla_cli ."
    echo ""
    exit 1
fi

# Make it executable
chmod +x ./orunla_cli

# Check if port 3000 is already in use
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "⚠️  Port 3000 is already in use"
    echo "Stopping existing server..."
    lsof -ti:3000 | xargs kill -9 2>/dev/null || true
    sleep 1
fi

# Start the server in the background
echo "🚀 Starting Orunla server on http://localhost:3000"
echo ""
./orunla_cli serve --port 3000 &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to initialize..."
sleep 3

# Check if server started successfully
if ! lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "❌ Failed to start server"
    exit 1
fi

# Open browser
echo "✅ Server running at http://localhost:3000"
echo ""
if command -v open > /dev/null; then
    # macOS
    open http://localhost:3000
elif command -v xdg-open > /dev/null; then
    # Linux
    xdg-open http://localhost:3000
else
    echo "Please open http://localhost:3000 in your browser"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Press Ctrl+C to stop the server"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo "🛑 Stopping server..."
    kill $SERVER_PID 2>/dev/null || true
    echo "✅ Server stopped"
    exit 0
}

# Trap Ctrl+C
trap cleanup INT TERM

# Wait for server process
wait $SERVER_PID

#!/bin/bash

# Script to run web server and 5 workers

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "Building web server..."
cargo build --release

echo "Building workers..."
cd workers
cargo build --release
cd ..

echo "Starting web server..."
# Run web server in background
./target/release/fin-dashboard &
SERVER_PID=$!
echo "Web server started with PID: $SERVER_PID"

# Wait a moment for server to be ready
sleep 2

echo "Starting 5 workers..."
WORKER_PIDS=()

# Start 5 workers
for i in {1..5}; do
    ./workers/target/release/b0t &
    WORKER_PID=$!
    WORKER_PIDS+=($WORKER_PID)
    echo "Worker $i started with PID: $WORKER_PID"
done

echo "============================================"
echo "Web server PID: $SERVER_PID"
echo "Worker PIDs: ${WORKER_PIDS[@]}"
echo "============================================"
echo "All processes started. Press Ctrl+C to stop all."

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "Stopping all processes..."

    # Stop workers
    for pid in "${WORKER_PIDS[@]}"; do
        echo "Killing worker PID: $pid"
        kill $pid 2>/dev/null
    done

    # Stop server
    echo "Killing server PID: $SERVER_PID"
    kill $SERVER_PID 2>/dev/null

    # Wait for processes to exit
    wait $SERVER_PID 2>/dev/null
    for pid in "${WORKER_PIDS[@]}"; do
        wait $pid 2>/dev/null
    done

    echo "All processes stopped."
    exit 0
}

# Trap Ctrl+C and call cleanup
trap cleanup SIGINT SIGTERM

# Wait for all background processes
wait

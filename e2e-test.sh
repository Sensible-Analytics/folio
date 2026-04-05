#!/bin/bash
# Download artifacts from latest successful CI run and run E2E tests

set -e

echo "Fetching latest successful PR Check run..."
RUN_ID=$(gh run list --branch bank-parser-demo --limit 10 --json databaseId,name,conclusion,workflowName | jq -r '.[] | select(.name == "PR Check" and .conclusion == "success") | .databaseId' | head -1)

if [ -z "$RUN_ID" ]; then
  echo "No successful PR Check run found. Exiting."
  exit 1
fi

echo "Downloading artifacts from run $RUN_ID..."
gh run download $RUN_ID --dir ./artifacts

# Check if artifacts exist
if [ ! -f "./artifacts/sensible-folio-server/sensible-folio-server" ]; then
  echo "Server binary not found in artifacts."
  exit 1
fi

if [ ! -d "./artifacts/frontend-build" ]; then
  echo "Frontend build not found in artifacts."
  exit 1
fi

# Make server binary executable
chmod +x ./artifacts/sensible-folio-server/sensible-folio-server

# Set environment variables for server
export WF_SECRET_KEY="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
export WF_DB_PATH="/tmp/test.db"
export WF_STATIC_DIR="./artifacts/frontend-build"

# Start server in background
echo "Starting server..."
./artifacts/sensible-folio-server/sensible-folio-server &
SERVER_PID=$!

# Wait for server to start
sleep 5

# Run E2E tests using existing script
echo "Running E2E tests..."
pnpm test:e2e

# Kill server
kill $SERVER_PID

echo "E2E tests completed."
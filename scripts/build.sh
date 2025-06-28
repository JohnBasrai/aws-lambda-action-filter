#!/bin/bash
set -euo pipefail

# Build and start the container
docker compose up -d --build

echo "Waiting for Lambda runtime to become ready..."
for i in {1..20}; do
  if nc -z localhost 9000; then
    echo "✅ Lambda runtime is up."
    break
  fi
  echo "[$i] ...still waiting"
  sleep 3
done

# Fail CI cleanly if it never becomes ready
if ! nc -z localhost 9000; then
  echo "❌ Lambda runtime did not become ready in time."
  docker ps -a
  docker logs aws-lambda-action-filter-lambda-1 || true
  exit 1
fi


# Run all tests inside the container
docker exec aws-lambda-action-filter-lambda-1 cargo fmt --version
docker exec aws-lambda-action-filter-lambda-1 cargo fmt --check
docker exec aws-lambda-action-filter-lambda-1 cargo build --release --quiet
docker exec aws-lambda-action-filter-lambda-1 cargo test --release --quiet

echo "All tests passed!"

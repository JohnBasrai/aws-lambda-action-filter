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

RUN="docker exec aws-lambda-action-filter-lambda-1"

# Run all tests inside the container
${RUN} cargo fmt --version
${RUN} cargo fmt --check
${RUN} cargo clippy --quiet --all-features -- -D warnings
${RUN} cargo build --release --quiet
${RUN} cargo test --release --quiet -- --nocapture

echo "All tests passed!"

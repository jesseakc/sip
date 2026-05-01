#!/bin/bash
set -euo pipefail

echo "=== SIP Preflight Check ==="
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "FAIL: Docker is not installed"
    exit 1
fi
echo "PASS: Docker found ($(docker --version))"

# Check Docker Compose
if ! docker compose version &> /dev/null; then
    echo "FAIL: Docker Compose is not available"
    exit 1
fi
echo "PASS: Docker Compose found"

# Check ports
for port in 3000 8000 5432 6379 9000; do
    if ss -tln | grep -q ":$port "; then
        echo "WARN: Port $port is in use"
    else
        echo "PASS: Port $port is free"
    fi
done

# Check .env exists
if [ ! -f .env ]; then
    if [ -f .env.example ]; then
        echo "WARN: No .env file. Run: cp .env.example .env"
    else
        echo "FAIL: No .env or .env.example found"
        exit 1
    fi
else
    echo "PASS: .env file exists"
fi

# Check JWT secret
if grep -q "change-me-in-production" .env 2>/dev/null; then
    echo "WARN: JWT secret is the default placeholder. Change for production."
fi

# Check Docker Compose config
if docker compose config &> /dev/null; then
    echo "PASS: docker compose config valid"
else
    echo "FAIL: docker compose config has errors"
    docker compose config 2>&1
fi

echo ""
echo "=== Preflight complete ==="

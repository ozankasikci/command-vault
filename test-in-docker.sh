#!/bin/bash

# Build and run Ubuntu tests
echo "Running Ubuntu tests..."
docker build -t command-vault-ubuntu -f Dockerfile.ubuntu .
docker run --rm -e COMMAND_VAULT_TEST=1 command-vault-ubuntu

# Build and run Windows tests
echo "Running Windows tests..."
docker build -t command-vault-windows -f Dockerfile.windows .
docker run --rm -e COMMAND_VAULT_TEST=1 command-vault-windows

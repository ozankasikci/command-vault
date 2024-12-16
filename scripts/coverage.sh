#!/bin/bash

# Install cargo-tarpaulin if not installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage

echo "Coverage report generated in coverage/tarpaulin-report.html"

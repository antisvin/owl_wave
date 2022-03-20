#!/bin/bash
set -eu

# Pre-requisites:
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version=0.2.79
cargo update -p wasm-bindgen

# For local tests with `./start_server`:
cargo install basic-http-server

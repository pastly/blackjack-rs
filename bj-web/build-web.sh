#!/usr/bin/env bash
set -eu

cargo fmt
cargo clippy
cargo check
wasm-pack -v build --dev --target web
./format-web-templates.py

#!/usr/bin/env bash
set -eu
cargo fmt
cargo clippy
wasm-pack -v build --dev --target web

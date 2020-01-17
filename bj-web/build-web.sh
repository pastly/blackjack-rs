#!/usr/bin/env bash
set -eu
cargo fmt
wasm-pack -v build --dev --target web

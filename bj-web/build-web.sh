#!/usr/bin/env bash
set -eu

if [[ "$#" != "1" ]] || [[ "$1" != "--release" ]]; then
	MODE="--dev"
else
	MODE="--release"
fi
echo "Compiling with $MODE"
cargo fmt
cargo clippy
cargo check
wasm-pack -v build $MODE --target web
./format-web-templates.py

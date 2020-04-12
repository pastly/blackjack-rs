#!/usr/bin/env bash
set -eu

if [[ "$#" != "1" ]] || [[ "$1" != "--release" ]]; then
	MODE="--dev"
else
	MODE="--release"
fi
echo "Compiling with $MODE"
for CRATE_CARGO in ./*/Cargo.toml; do
    CRATE_DIR=$(dirname $CRATE_CARGO)
    cd $CRATE_DIR
    cargo fmt
    cargo clippy
    cargo check
    wasm-pack -v build $MODE --target web
    cd -
done
./format-web-templates.py

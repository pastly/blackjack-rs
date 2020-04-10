#!/usr/bin/env bash
set -eu

if [[ "$#" != "1" ]] || [[ "$1" != "--release" ]]; then
	MODE="--dev"
else
	MODE="--release"
fi
echo "Compiling with $MODE"
for CRATE in index; do
    cd $CRATE
    cargo fmt
    cargo clippy
    cargo check
    wasm-pack -v build $MODE --target web
    cd -
done
./format-web-templates.py

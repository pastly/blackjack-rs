#!/usr/bin/env bash
set -eu
function inject_version() {
    # BEGIN EDIT
    SRC=www/index.html
    DST=www/index.html
    # END EDIT
    TMP=$(mktemp)
    COMMIT=$(git rev-parse --short HEAD)
    DATE=$(git show -s --format='%cd' --date='format:%Y-%m-%d' $COMMIT)
    VERSION_LINE="<!-- VERSION_LINE -->Version $DATE ($COMMIT)"
    sed "s|^.*VERSION_LINE.*$|$VERSION_LINE|g" $SRC > $TMP
    cp -v $TMP $DST
    rm $TMP
}

cargo fmt
cargo clippy
cargo check
wasm-pack -v build --dev --target web
inject_version

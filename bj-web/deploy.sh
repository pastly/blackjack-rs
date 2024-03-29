#!/usr/bin/env bash
set -eu
./build-web.sh --release
for A in basic-strategy/pkg/*.{js,wasm} time-trial/pkg/*.{js,wasm} counting/pkg/*.{js,wasm}
do
    aws s3 cp $A s3://blackjack-wasm/pub/$(basename $A)
done

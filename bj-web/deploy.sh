#!/usr/bin/env bash
set -eu
./build-web.sh --release
for A in free-play/pkg/bj_web_free_play_bg.wasm \
        free-play/pkg/bj_web_free_play.js; do
    aws s3 cp $A s3://blackjack-wasm/pub/$(basename $A)
done

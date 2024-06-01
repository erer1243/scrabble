#!/bin/sh -xe
cd "$(dirname "$0")"
(cd web && npm run dev -- --host) &
NPM="$!"
cargo watch --exec run --ignore web &
CARGO="$!"
trap 'kill $NPM $CARGO' EXIT INT
wait

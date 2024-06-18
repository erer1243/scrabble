#!/bin/sh -xe
cd "$(dirname "$0")"
(cd web && npm install && npm run dev -- --host >/dev/null 2>&1) &
NPM="$!"
cargo watch --exec run --watch game --watch server --watch Cargo.toml --watch Cargo.lock --clear &
CARGO="$!"
trap '{ pkill -f target/debug/server; kill $NPM $CARGO; } >/dev/null 2>&1' EXIT INT
wait

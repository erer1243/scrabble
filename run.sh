#!/bin/sh -xe
cd "$(dirname "$0")"

cd web
npm run build
npm run preview -- --host >/dev/null 2>&1 &
NPM="$!"

cd ..
cargo build --release
./target/release/server &
SRV="$!"

trap '{ kill $NPM; kill $SRV; } >/dev/null 2>&1' EXIT INT
wait

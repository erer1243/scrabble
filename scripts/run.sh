#!/bin/sh -xe
cd "$(dirname "$0")"/..

cd web
npm run build
npm run preview -- --host >/dev/null 2>&1 &
NPM1="$!"

npm run dev >/dev/null 2>&1 &
NPM2="$!"

cd ..
cargo build --release
./target/release/server &
SRV="$!"

trap '{ kill $NPM1; kill $SRV; kill $NPM2; } >/dev/null 2>&1' EXIT INT
wait

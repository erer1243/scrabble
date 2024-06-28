#!/usr/bin/env bash
set -xe
cd "$(dirname "$0")"/..

cargo build --release &

cd web
npm run build &
cd ..

wait
rsync -av ./target/release/server uncle.onet:scrabble/server
rsync -av ./web/dist/ uncle.onet:scrabble/dist/
rsync -av ./scripts/run.sh uncle.onet:scrabble/run.sh

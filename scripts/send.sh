#!/usr/bin/env bash
set -xe
cd "$(dirname "$0")"/..

cargo build --release &

cd web
npm run build &
cd ..

wait
SSH='ssh -o ControlPath=ssh-%C'
$SSH -Nf -o ControlMaster=yes uncle.onet
$SSH uncle.onet mkdir -p scrabble
rsync -e "$SSH" -av ./target/release/server uncle.onet:scrabble/server
rsync -e "$SSH" -av ./web/dist/ uncle.onet:scrabble/dist/
rsync -e "$SSH" -av ./scripts/run.sh uncle.onet:scrabble/run.sh
$SSH -O exit uncle.onet

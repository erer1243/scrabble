#!/usr/bin/env bash
set -xe
cd "$(dirname "$0")"/..

cargo build --release &

cd web
npm run build &
cd ..

wait
ssh -Nf -o ControlMaster=yes -o ControlPath="ssh-%C" uncle.onet
rsync -e 'ssh -o ControlPath="ssh-%C"' -av ./target/release/server uncle.onet:scrabble/server
rsync -e 'ssh -o ControlPath="ssh-%C"' -av ./web/dist/ uncle.onet:scrabble/dist/
rsync -e 'ssh -o ControlPath="ssh-%C"' -av ./scripts/run.sh uncle.onet:scrabble/run.sh
ssh -O exit -o ControlPath="ssh-%C" uncle.onet

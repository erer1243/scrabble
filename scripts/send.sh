#!/usr/bin/env bash
set -xe
cd "$(dirname "$0")"/..

pushd web
npm run build &
popd

SSH='ssh -o ControlPath=ssh-%C'
$SSH -Nf -o ControlMaster=yes uncle.onet
$SSH uncle.onet mkdir -p scrabble

DEPLOYED_VERSION="$($SSH uncle.onet 'test -x scrabble/server && scrabble/server --version')"
LATEST_VERSION="$(git rev-parse --short HEAD)"

if [ "$DEPLOYED_VERSION" != "$LATEST_VERSION" ]; then
  cargo build --release
  rsync -e "$SSH" -av ./target/release/server uncle.onet:scrabble/server
fi

wait

rsync -e "$SSH" -av ./web/dist/ uncle.onet:scrabble/dist/
rsync -e "$SSH" -av ./scripts/run.sh uncle.onet:scrabble/run.sh
$SSH -O exit uncle.onet

#!/usr/bin/env bash
set -xe
cd "$(dirname "$0")"/..

SSH='ssh -o ControlPath=ssh-%C'
$SSH -Nf -o ControlMaster=yes uncle.onet
$SSH uncle.onet mkdir -p scrabble
rsync -e "$SSH" -av ./scripts/run.sh uncle.onet:scrabble/run.sh
rsync -e "$SSH" -av ./scripts/digest.sh uncle.onet:scrabble/digest.sh

# build & send server binary
DEPLOYED_VERSION="$($SSH uncle.onet 'test -x scrabble/server && scrabble/server --version')"
LATEST_VERSION="$(git rev-parse --short HEAD)"
if [ "$DEPLOYED_VERSION" != "$LATEST_VERSION" ]; then
  cargo build --release
  rsync -e "$SSH" -av ./target/release/server uncle.onet:scrabble/server
fi

# build & send web assets
DEPLOYED_SUM="$($SSH uncle.onet 'test -d scrabble/dist && scrabble/digest.sh scrabble/dist')"
LOCAL_SUM="$(test -d web/dist && scripts/digest.sh web/dist)"
if { ! test -d web/dist; } || [ "$DEPLOYED_SUM" != "$LOCAL_SUM" ]; then
  pushd web
  npm run build
  popd
  rsync -e "$SSH" -av --delete ./web/dist/ uncle.onet:scrabble/dist/
fi

$SSH -O exit uncle.onet

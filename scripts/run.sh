#!/bin/sh -xe
if [ "$(hostname)" = uncle ]; then
  if [ "$TMUX" ]; then
    cd "$HOME"/scrabble
    ./server &
    SRV=$!
    cd dist
    python -m http.server 4173 &
    WEB=$!
    trap 'kill $SRV; kill $WEB' EXIT
    wait
  else
    if ! (tmux list-sessions -F '#{session_name}' 2>/dev/null | grep -q '^scrabble$'); then
      tmux new-session -s scrabble "$(realpath "$0")"
    else
      tmux attach-session -t scrabble
    fi
  fi
else
  ssh -t uncle.onet './scrabble/run.sh'
fi

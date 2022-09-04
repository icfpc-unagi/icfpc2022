#!/bin/bash -eu

function submit() {
  echo
  echo "Submitting $2"
  curl -s -X POST --data-urlencode isl@$2 -d problem_id=$1 https://icfpc.sx9.jp/scvzcaae/internal_submit
  rm "$2"
}

if (($# > 0)); then
  nice cargo run --release --bin optimize -- -a --submission-ids "$*"
else
  nice cargo run --release --bin optimize
  # nice cargo run --release --bin optimize -- -a -l 100
  # nice cargo run --release --bin optimize -- -a --program-name chokudai
fi

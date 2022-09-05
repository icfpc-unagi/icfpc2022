#!/bin/bash -eu

function submit() {
  echo
  echo "Submitting $2" >&2
  curl -s -X POST --data-urlencode isl@$2 -d problem_id=$1 https://icfpc.sx9.jp/scvzcaae/internal_submit
  # TODO: determine internal submission-id and record
  rm "$2"
}

# for i in {1..40}; do
# 1 位取れてない問題
for i in 1 3 10 11 13 14 15 16 17 19 20 21 23 25 28 38; do
  echo "==================== $i ===================="
  f="out/$i-$(date +%s).isl"
  git pull
  RAYON_NUM_THREADS=200 FLIP_ROTATE=1 MAX_CANDIDATES=150 MAX_WIDTH=100 nice -- cargo run --release --bin wata5 -- "$i" | tee "$f"
  if [[ -s $f ]]; then
    submit "$i" "$f"
  else
    rm $f
  fi
done

#!/bin/bash -eu

function submit() {
  echo
  echo "Submitting $2" >&2
  curl -s -X POST --data-urlencode isl@$2 -d problem_id=$1 https://icfpc.sx9.jp/scvzcaae/submit
  # TODO: determine internal submission-id and record
  rm "$2"
}

# for i in {1..40}; do
# 1 位取れてない問題
for i in 1 3 6 7 9 10 11 13 14 16 17 19 21 24 25 28 30 35; do
  echo "==================== $i ===================="
  f="out/$i-$(date +%s).isl"
  RAYON_NUM_THREADS=200 FLIP_ROTATE=1 MAX_CANDIDATES=150 MAX_WIDTH=100 nice -n5 -- cargo run --release --bin wata5 -- "$i" | tee "$f"
  if [[ -s $f ]]; then
     nice -n5 -- cargo run --release --bin optimize_file -- --problem-id "$i" --program "$f" --out "opt_$f"
    if [[ -s "opt_$f" ]]; then
      submit "$i" "opt_$f"
    else
      submit "$i" "$f"
    fi
  else
    rm $f
  fi
done

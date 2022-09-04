#!/bin/bash -eu

function submit() {
  echo
  echo "Submitting $2" >&2
  curl -s -X POST --data-urlencode isl@$2 -d problem_id=$1 https://icfpc.sx9.jp/scvzcaae/internal_submit
  # TODO: determine internal submission-id and record
  rm "$2"
}

# TODO: initial png できたら 40 に
for i in {1..40}; do
  echo "==================== $i ===================="
  f="out/$i-$(date +%s).isl"
  git pull
  RAYON_NUM_THREADS=200 FLIP_ROTATE=1 MAX_CANDIDATES=150 MAX_WIDTH=50 nice cargo build --release --bin wata5 -- $i | tee $f
  if [[ -s $f ]]; then
    submit "$i" "$f"
  else
    rm $f
  fi
done

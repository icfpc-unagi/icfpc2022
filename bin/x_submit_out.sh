#!/bin/bash -eu

function submit() {
  echo
  echo "Submitting $2"
  curl -s -X POST --data-urlencode isl@$2 -d problem_id=$1 https://icfpc.sx9.jp/scvzcaae/internal_submit
  rm "$2"
}

for i in out/*.isl; do 
  if [[ -s $i ]]; then
    f=$(basename ${i%-*})
    submit "$f" "$i"
  fi
done

for i in out/opt_*_*; do
  if [[ -s $i ]]; then
    unprefix=${i#out/opt_}
    f=${unprefix%_*}
    submit "$f" "$i"
  fi
done

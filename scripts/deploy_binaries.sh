#!/usr/bin/env bash

version="$1"

if [ "${version}" == '' ]; then
    echo "Version ID must be given in the first argument" >&2
    exit 1
fi

pushd /work/src/bin
targets=()
for file in *.rs; do
    targets+=("/work/target/release/${file//.rs}")
done
popd >/dev/null

gsutil -m cp "${targets[@]}" gs://icfpc2022/${version}/

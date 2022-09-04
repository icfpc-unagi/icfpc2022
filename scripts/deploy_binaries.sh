#!/usr/bin/env bash

version="$1"

if [ "${version}" == '' ]; then
    echo "Version ID must be given in the first argument" >&2
    exit 1
fi

gsutil -m cp /usr/local/bin/* gs://icfpc2022/bin/${version}/

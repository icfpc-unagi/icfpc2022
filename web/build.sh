#!/bin/sh

set -ex

mkdir -p www
wasm-pack build --target no-modules
rm -f www/*
cp index.html www/
cp pkg/*.js www/
cp pkg/*.wasm www/
cd www
python3 -m http.server

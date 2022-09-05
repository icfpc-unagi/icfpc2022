.PHONY: all wasm-pack build run test clean

all: wasm-pack build run

wasm-pack:
	wasm-pack build --target web

build:
	npm install
	npm run build

run:
	npm run start

test:
	npm run test

clean:
	rm -rf dist

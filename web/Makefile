.PHONY: wasm-pack build run
build: index.html pkg/web_bg.wasm
	mkdir -p www
	rm -f www/*
	cp index.html pkg/*.js pkg/*.wasm www/
run: build
	cd www && python3 -m http.server
pkg/web_bg.wasm: src/lib.rs
	wasm-pack build --target no-modules

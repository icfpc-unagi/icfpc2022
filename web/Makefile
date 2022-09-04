.PHONY: wasm-pack build run
pkg/*: src/*
	wasm-pack build --target no-modules
build: src/* wasm-pack
	mkdir -p www
	rm -f www/*
	cp index.html www/
	cp pkg/*.js www/
	cp pkg/*.wasm www/
run: build
	cd www && python3 -m http.server

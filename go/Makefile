.PHONY: test
test:
	go test ./...

.PHONY: run
run:
	cd .. && make run/server

.PHONY: format
format:
	go fmt ./...

.PHONY: push
push:
	cd .. && make push/server
	gcloud --project=icfpc-primary run deploy server \
		--region=asia-northeast1 \
		--image=asia-docker.pkg.dev/icfpc-primary/asia/server

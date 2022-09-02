%: %.Dockerfile
	$(MAKE) $*@build

server: server.Dockerfile
	cd .. && make secrets
	$(MAKE) server@build

%@build: FORCE
	cd .. && docker build --build-arg="UNAGI_PASSWORD=${UNAGI_PASSWORD}" \
		-f docker/$*.Dockerfile -t icfpc-unagi/$* .

.PHONY: FORCE
FORCE:

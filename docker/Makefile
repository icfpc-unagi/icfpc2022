%: %.Dockerfile
	cd .. && docker build --build-arg="UNAGI_PASSWORD=${UNAGI_PASSWORD}" \
		-f docker/$*.Dockerfile -t icfpc-unagi/$* .

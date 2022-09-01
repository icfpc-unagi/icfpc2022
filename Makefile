.PHONY: test
test: test/rust

.PHONY: test/rust
test/rust:
	cargo test

check:
	@bash ./scripts/check_unagi_password.sh --logtostderr
	@echo 'Successfully passed precondition check.' >&2

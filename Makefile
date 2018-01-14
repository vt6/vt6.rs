build: FORCE
	cargo build

test: FORCE
	cargo test
check: test FORCE

.PHONY: FORCE

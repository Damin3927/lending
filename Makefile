.PHONY: test
test:
	anchor test --skip-local-validator -- --features anchor-test

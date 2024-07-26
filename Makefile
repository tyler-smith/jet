export LLVM_SYS_180_PREFIX=/usr/local/opt/llvm
export RUST_BACKTRACE=1

.PHONY: format
format: ## Format the code
	rustfmt src/**/*.rs

.PHONY: commit-check
commit-check: format ## Full check to run before commits
	cargo check
	cargo clippy --all-targets --all-features -- -D warnings

.DEFAULT_GOAL := help
.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

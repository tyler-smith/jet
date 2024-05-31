export LLVM_SYS_180_PREFIX=/usr/local/opt/llvm
export RUST_BACKTRACE=1

BIN_PATH ?= /usr/local/opt/llvm/bin
CLANG_BIN ?= $(BIN_PATH)/clang

#runtime: dist  ## Build the runtime
#$(CLANG_BIN) -dynamiclib -o dist/runtime.dylib ./src/runtime/runtime.c

.PHONY: runtime
runtime: dist ## Build the runtime
	mkdir -p build
	(cd build && cmake .. -GNinja)
	(cd build && ninja)
	cp build/dist/* dist/
	#$(CLANG_BIN) -S -emit-ir -o dist/runtime.dylib ./runtime/runtime.c
	#$(CLANG_BIN) -dynamiclib -o dist/runtime.dylib ./runtime/runtime.c
	#rustc --crate-type lib --emit=llvm-ir -o ./dist/jetvm-runtime.ll ./crates/runtime/src/lib.rs

.PHONY: check
check: ## Check the project
	cargo check

.PHONY: build
build: ## Build the project
	cargo build

.PHONY: run
run: ## Run the jetdbg
	cargo run

.PHONY: test
test: ## Run package tests
	cargo nextest run

.PHONY: fmt
fmt: format ## Format Rust code

.PHONY: format
format:
	rustfmt src/**/*.rs

dist:
	mkdir -p dist

.PHONY: clean
clean: ## Clean the compiled objects
	rm -rf ./target ./dist

.DEFAULT_GOAL := help
.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

SHELL = /bin/bash

.PHONY: check clean help

## check: Check code and style.
check:
	@cargo clippy -- -D clippy::all
	@cargo fmt --all -- --check

## clean: Clean all build files.
clean:
	@rm -rf $(OUTPUT_DIR)
	@cargo clean

## help: Show this help.
help: Makefile
	@echo Usage: make [command]
	@sed -n 's/^##//p' $< | column -t -s ':' |  sed -e 's/^/ /'

BIN = target/release/scandisk

.PHONY: help debug build scan carve carve-all carve-pdf carve-png carve-jpeg test clean fmt clippy check

## help: Show this help message
help:
	@echo "Usage: [sudo] make [target] [TARGET=<path>] [OUT=<dir>] [TYPE=<types>]"
	@echo ""
	@echo "Targets:"
	@sed -n 's/^## //p' $(MAKEFILE_LIST) | column -t -s ':' | sed -e 's/^/  /'

## debug: Compile debug binary
debug:
	cargo build

## build: Compile optimized release binary
build:
	cargo build --release

## scan: Scan target (TARGET=<path> [TYPE=png,jpeg])
scan:
	./$(BIN) $(TARGET) $(if $(TYPE),-t $(TYPE))

## carve: Scan and extract files (TARGET=<path> OUT=<dir> [TYPE=png,jpeg])
carve:
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) $(if $(TYPE),-t $(TYPE))

## carve-all: Scan and extract all file types (TARGET=<path> OUT=<dir>)
carve-all:
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) -t all

## carve-pdf: Scan and extract all pdf files (TARGET=<path> OUT=<dir>)
carve-pdf:
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) -t pdf

## carve-png: Scan and extract all png files (TARGET=<path> OUT=<dir>)
carve-png:
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) -t png

## carve-jpeg: Scan and extract all jpeg files (TARGET=<path> OUT=<dir>)
carve-jpeg:
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) -t jpeg

## test: Run cargo unit tests
test:
	cargo test

## clean: Remove cargo target directory
clean:
	cargo clean

## fmt: Format the code
fmt:
	cargo fmt

## clippy: Strict linter, fails on warning and suggests fixes
clippy:  fmt
	cargo clippy -- -D warnings 

## check: Check for errors without building
check: clippy
	cargo check
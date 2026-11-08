BIN          = target/release/scandisk
DOCKER_IMAGE = scandisk

.PHONY: help debug build scan carve carve-all carve-pdf carve-png carve-jpeg test clean fmt clippy check docker-build docker-scan docker-carve

## help: Show this help message
help:
	@echo "Usage: [sudo] make [target] [TARGET=<path>] [OUT=<dir>] [TYPE=<types>]"
	@echo ""
	@echo "Targets:"
	@sed -n 's/^## //p' $(MAKEFILE_LIST) | column -t -s ':' | sed -e 's/^/  /'

# ==================== Native Targets (Requires Rust) ====================

## debug: Compile debug binary
debug:
	cargo build

## build: Compile optimized release binary
build:
	cargo build --release

## scan: Scan target natively (TARGET=<path> [TYPE=png,jpeg])
scan: build
	./$(BIN) $(TARGET) $(if $(TYPE),-t $(TYPE))

## carve: Scan and extract files natively (TARGET=<path> OUT=<dir> [TYPE=png,jpeg])
carve: build
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) $(if $(TYPE),-t $(TYPE))

## carve-all: Scan and extract all file types (TARGET=<path> OUT=<dir>)
carve-all: build
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) -t all

## carve-pdf: Scan and extract all pdf files (TARGET=<path> OUT=<dir>)
carve-pdf: build
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) -t pdf

## carve-png: Scan and extract all png files (TARGET=<path> OUT=<dir>)
carve-png: build
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) -t png

## carve-jpeg: Scan and extract all jpeg files (TARGET=<path> OUT=<dir>)
carve-jpeg: build
	mkdir -p $(OUT)
	./$(BIN) $(TARGET) -o $(OUT) -t jpeg

## test: Run cargo unit tests
test: debug
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

# ==================== Docker Targets ====================

# Automatically use 'docker compose' if available, otherwise fall back to 'docker-compose' (legacy)
DOCKER_COMPOSE := $(shell if docker compose version >/dev/null 2>&1; then echo "docker compose"; else echo "docker-compose"; fi)

## docker-build: Force a rebuild of the container image
docker-build:
	$(DOCKER_COMPOSE) build

## docker-scan: Scan target via Docker (TARGET=<path> [TYPE=png,jpeg])
docker-scan:
	$(DOCKER_COMPOSE) run --rm scandisk $(TARGET) $(if $(TYPE),-t $(TYPE))

## docker-carve: Carve target via Docker (TARGET=<path> OUT=<dir> [TYPE=png,jpeg])
docker-carve:
	mkdir -p $(OUT)
	$(DOCKER_COMPOSE) run --rm scandisk $(TARGET) -o $(OUT) $(if $(TYPE),-t $(TYPE))
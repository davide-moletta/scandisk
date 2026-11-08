# Fetch rust container to build the binary
FROM rust:slim AS builder

WORKDIR /usr/src/scandisk

# Copy source files
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# Build release image
RUN cargo build --release

# Fetch Debian 12 image
FROM debian:bookworm-slim

# Copy the release binary from the builder image to the debian one (results in smaller container)
COPY --from=builder /usr/src/scandisk/target/release/scandisk /usr/local/bin/scandisk

WORKDIR /data

ENTRYPOINT ["/usr/local/bin/scandisk"]
CMD ["--help"]
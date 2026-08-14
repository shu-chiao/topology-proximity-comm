FROM rust:1-bookworm AS builder

WORKDIR /build
COPY rust ./rust
COPY configs ./configs

WORKDIR /build/rust
RUN cargo build --release --bin main_pub --bin main_sub

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/rust/target/release/main_pub /usr/local/bin/main_pub
COPY --from=builder /build/rust/target/release/main_sub /usr/local/bin/main_sub
COPY configs /build/configs
COPY docker/rust/run-demo.sh /app/run-demo.sh

WORKDIR /app
RUN chmod +x /app/run-demo.sh

CMD ["/app/run-demo.sh"]

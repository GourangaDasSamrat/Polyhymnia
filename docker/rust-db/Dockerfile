FROM rust:latest AS builder

WORKDIR /usr/src/rust-db
COPY rust-db /usr/src/rust-db
COPY proto /usr/src/proto

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
	protobuf-compiler ca-certificates && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rust-db/target/release/rust-db /usr/local/bin/rust-db
EXPOSE 50051
CMD ["/usr/local/bin/rust-db"]

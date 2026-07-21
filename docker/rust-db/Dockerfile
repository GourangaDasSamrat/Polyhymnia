FROM rust:latest AS builder

WORKDIR /usr/src/rust-db
COPY rust-db /usr/src/rust-db

RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rust-db/target/release/rust-db /usr/local/bin/rust-db
EXPOSE 50051
CMD ["/usr/local/bin/rust-db"]

# Polyhymnia — build and run every service in the pipeline.
#
# Requirements: protoc, protoc-gen-go, protoc-gen-go-grpc, grpc_cpp_plugin,
# a C++17 toolchain + gRPC/Protobuf dev libraries, Rust/cargo, Go 1.22+,
# and Python 3 (only used to serve the static frontend).

default:
    @just --list

# ---- codegen -----------------------------------------------------------

# Generate Go protobuf/gRPC stubs into go-gateway/proto.
proto-go:
    protoc \
        --go_out=go-gateway/proto --go_opt=paths=source_relative \
        --go-grpc_out=go-gateway/proto --go-grpc_opt=paths=source_relative \
        --proto_path=proto proto/quote.proto

# C++ codegen happens automatically as part of `build-cpp` via CMake,
# this target is only for inspecting the generated code manually.
proto-cpp:
    mkdir -p cpp-engine/build/generated
    protoc \
        --cpp_out=cpp-engine/build/generated \
        --grpc_out=cpp-engine/build/generated \
        --plugin=protoc-gen-grpc=$(which grpc_cpp_plugin) \
        --proto_path=proto proto/quote.proto

# Rust codegen happens automatically via build.rs on `cargo build`.

proto: proto-go

# ---- build --------------------------------------------------------------

build-rust:
    cd rust-db && cargo build --release

build-cpp:
    cmake -B cpp-engine/build -S cpp-engine -DCMAKE_BUILD_TYPE=Release
    cmake --build cpp-engine/build

build-go: proto-go
    cd go-gateway && go mod tidy && go build -o bin/gateway .

build: build-rust build-cpp build-go

# ---- run (each in its own terminal) --------------------------------------

run-rust:
    cd rust-db && cargo run --release

run-cpp:
    ./cpp-engine/build/cpp-engine

run-go:
    cd go-gateway && go run .

run-frontend:
    cd frontend && python3 -m http.server 5500

# ---- run everything at once ----------------------------------------------

# Builds everything, then launches all four processes concurrently.
# Ctrl+C once to stop all of them.
run: build
    #!/usr/bin/env bash
    set -euo pipefail
    trap 'kill 0' EXIT

    (cd rust-db && cargo run --release) &
    ./cpp-engine/build/cpp-engine &
    sleep 3
    (cd go-gateway && go run .) &
    (cd frontend && python3 -m http.server 5500) &

    echo "Frontend:   http://localhost:5500"
    echo "Gateway:    http://localhost:8080/api/quote"
    echo "Rust gRPC:  localhost:50051"
    echo "C++ gRPC:   localhost:50052"

    wait

clean:
    rm -rf cpp-engine/build go-gateway/bin go-gateway/proto/*.pdb.go rust-db/target rust-db/quotes.db

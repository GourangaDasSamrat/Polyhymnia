FROM ubuntu:22.04 AS builder

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential cmake pkg-config protobuf-compiler libprotobuf-dev libgrpc++-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src
COPY cpp-engine ./cpp-engine
COPY proto ./proto

RUN cmake -B build -S ./cpp-engine -DCMAKE_BUILD_TYPE=Release && cmake --build build --config Release

FROM ubuntu:22.04
COPY --from=builder /usr/src/cpp-engine/build/cpp-engine /usr/local/bin/cpp-engine
EXPOSE 50052
CMD ["/usr/local/bin/cpp-engine"]

FROM ubuntu:22.04 AS builder

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential cmake pkg-config protobuf-compiler protobuf-compiler-grpc libprotobuf-dev libgrpc++-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src
COPY cpp-engine ./cpp-engine
COPY proto ./proto

RUN cmake -B build -S ./cpp-engine -DCMAKE_BUILD_TYPE=Release && cmake --build build --config Release

FROM ubuntu:22.04
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    ca-certificates libprotobuf-dev libgrpc++-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/build/cpp-engine /usr/local/bin/cpp-engine
EXPOSE 50052
CMD ["/usr/local/bin/cpp-engine"]

FROM golang:1.26 AS builder

WORKDIR /app
COPY go-gateway /app
COPY proto /proto
RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends protobuf-compiler ca-certificates \
 && rm -rf /var/lib/apt/lists/*

RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@v1.34.2 \
 && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@v1.5.1

ENV PATH="/root/go/bin:${PATH}"

RUN protoc \
	--go_out=/app/proto --go_opt=paths=source_relative \
	--go-grpc_out=/app/proto --go-grpc_opt=paths=source_relative \
	--proto_path=/proto /proto/quote.proto

RUN go mod tidy \
 && CGO_ENABLED=0 GOOS=linux go build -ldflags='-s -w' -o /app/bin/gateway .

FROM alpine:3.18
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/bin/gateway /usr/local/bin/gateway
EXPOSE 8080
CMD ["/usr/local/bin/gateway"]

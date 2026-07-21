FROM golang:1.26 AS builder

WORKDIR /app
COPY go-gateway /app
COPY proto /proto
RUN go mod tidy \
 && go install google.golang.org/protobuf/cmd/protoc-gen-go@latest \
 && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest \
 && CGO_ENABLED=0 GOOS=linux go build -ldflags='-s -w' -o /app/bin/gateway .

FROM alpine:3.18
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/bin/gateway /usr/local/bin/gateway
EXPOSE 8080
CMD ["/usr/local/bin/gateway"]

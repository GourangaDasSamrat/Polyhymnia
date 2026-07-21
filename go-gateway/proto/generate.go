// Package quotepb holds the generated protobuf/gRPC Go code for
// Polyhymnia. Nothing in this file is compiled into the binary; it only
// documents how the real quotepb/*.pb.go files are produced.
//
// This directory (go-gateway/proto) is import path "polyhymnia/proto",
// because the go-gateway module (module "polyhymnia", per go.mod) is
// rooted at go-gateway/, not at the repo root. main.go imports this
// package as `pb "polyhymnia/proto"` — keep proto/quote.proto's
// go_package option and this directory in sync with that if either one
// ever moves.
//
// Run `just proto-go` (see the repo's justfile) or the command below from
// the repo root to generate quote.pb.go and quote_grpc.pb.go into this
// directory:
//
//	protoc \
//	  --go_out=go-gateway/proto --go_opt=paths=source_relative \
//	  --go-grpc_out=go-gateway/proto --go-grpc_opt=paths=source_relative \
//	  --proto_path=proto proto/quote.proto
//
// Requires the protoc-gen-go and protoc-gen-go-grpc plugins:
//
//	go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
//	go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
package quotepb

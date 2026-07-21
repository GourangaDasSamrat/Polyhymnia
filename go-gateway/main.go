// Polyhymnia — Go "API Gateway / Orchestrator".
//
// Exposes a single HTTP endpoint to the frontend and coordinates the
// three-step gRPC workflow across the Rust and C++ services:
//
//  1. Rust  : GetAllIds        -> every quote ID in the database
//  2. C++   : SelectRandomId   -> one ID, chosen at "random"
//  3. Rust  : GetQuoteById     -> the quote text for that ID
package main

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "polyhymnia/proto"
)

// retryConfig controls connection retry behavior.
const (
	maxRetries     = 30           // Max number of retry attempts
	retryInterval  = 500 * time.Millisecond // Wait between retries
)

const (
	// Dialed as explicit IPv4 loopback rather than "localhost": on some
	// systems "localhost" resolves to 127.0.0.1 while a server bound to
	// "[::1]" (IPv6-only) is listening on a different address, causing
	// a misleading "connection refused". Both backend services bind
	// IPv4 addresses (see rust-db/src/main.rs and cpp-engine/src/main.cpp),
	// so dial the same family here.
	rustDbAddr    = "127.0.0.1:50051"
	cppEngineAddr = "127.0.0.1:50052"
	httpAddr      = ":8080"
	callTimeout   = 5 * time.Second
)

// gateway holds the gRPC clients used to talk to the downstream services.
type gateway struct {
	dbClient  pb.QuoteDbClient
	rngClient pb.RandomizerClient
}

type quotePayload struct {
	Quote  string `json:"quote"`
	Author string `json:"author"`
}

type errorPayload struct {
	Error string `json:"error"`
}

func dialGrpc(addr string) *grpc.ClientConn {
	var conn *grpc.ClientConn
	var lastErr error

	for attempt := 1; attempt <= maxRetries; attempt++ {
		conn, lastErr = grpc.NewClient(addr,
			grpc.WithTransportCredentials(insecure.NewCredentials()),
			grpc.WithBlock(),
		)

		if lastErr == nil {
			log.Printf("✓ Connected to %s on attempt %d", addr, attempt)
			return conn
		}

		if attempt < maxRetries {
			log.Printf("Connecting to %s (attempt %d/%d)... %v", addr, attempt, maxRetries, lastErr)
			time.Sleep(retryInterval)
		}
	}

	log.Fatalf("✗ Failed to connect to %s after %d attempts: %v", addr, maxRetries, lastErr)
	return nil
}

// getQuoteHandler runs the full orchestration described in the spec and
// returns the final quote as JSON.
func (g *gateway) getQuoteHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")

	if r.Method != http.MethodGet {
		writeError(w, http.StatusMethodNotAllowed, "method not allowed")
		return
	}

	ctx, cancel := context.WithTimeout(r.Context(), callTimeout)
	defer cancel()

	// Step 1: fetch every quote ID from the Rust database service.
	idList, err := g.dbClient.GetAllIds(ctx, &pb.Empty{})
	if err != nil {
		log.Printf("GetAllIds failed: %v", err)
		writeError(w, http.StatusBadGateway, "failed to fetch quote ids")
		return
	}
	if len(idList.GetIds()) == 0 {
		writeError(w, http.StatusNotFound, "no quotes available")
		return
	}

	// Step 2: hand the full ID list to the C++ engine and let it pick one.
	selected, err := g.rngClient.SelectRandomId(ctx, idList)
	if err != nil {
		log.Printf("SelectRandomId failed: %v", err)
		writeError(w, http.StatusBadGateway, "failed to select a random id")
		return
	}

	// Step 3: fetch the quote text + author for the chosen ID.
	quote, err := g.dbClient.GetQuoteById(ctx, &pb.QuoteRequest{Id: selected.GetId()})
	if err != nil {
		log.Printf("GetQuoteById failed: %v", err)
		writeError(w, http.StatusBadGateway, "failed to fetch quote text")
		return
	}

	// Step 4: return the assembled payload to the frontend.
	resp := quotePayload{Quote: quote.GetQuote(), Author: quote.GetAuthor()}
	if err := json.NewEncoder(w).Encode(resp); err != nil {
		log.Printf("failed to encode response: %v", err)
	}
}

func writeError(w http.ResponseWriter, status int, message string) {
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(errorPayload{Error: message})
}

func main() {
	dbConn := dialGrpc(rustDbAddr)
	defer dbConn.Close()

	rngConn := dialGrpc(cppEngineAddr)
	defer rngConn.Close()

	gw := &gateway{
		dbClient:  pb.NewQuoteDbClient(dbConn),
		rngClient: pb.NewRandomizerClient(rngConn),
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/api/quote", gw.getQuoteHandler)

	log.Printf("Go API Gateway listening on %s", httpAddr)
	if err := http.ListenAndServe(httpAddr, mux); err != nil {
		log.Fatalf("server failed: %v", err)
	}
}

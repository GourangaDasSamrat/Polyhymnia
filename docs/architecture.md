# Architecture

Polyhymnia deliberately spreads a single-row database lookup across
four services and three gRPC round-trips. This document explains why
each piece exists and how they fit together.

## Services

### 1. Frontend (JavaScript / HTML / CSS)

- Location: `frontend/`
- One button (`Get Quote`), one `<blockquote>`, one author line.
- On click, issues `GET http://localhost:8080/api/quote` and renders
  the JSON response. No frameworks, no build step.

### 2. Go API Gateway / Orchestrator

- Location: `go-gateway/`
- The only service exposed over plain HTTP; everything downstream is
  gRPC-only.
- Owns the orchestration logic end-to-end (see [Data flow](#data-flow)
  below) and is the single point where the three-step workflow is
  sequenced.
- Talks to both backend services as a gRPC **client**:
  `QuoteDbClient` (Rust) and `RandomizerClient` (C++).

### 3. C++ Mathematical Engine (Randomizer)

- Location: `cpp-engine/`
- Implements exactly one RPC: `SelectRandomId(IdList) -> SelectedId`.
- Deliberately overengineered per the project brief: combines two
  `std::random_device` draws into a 64-bit seed via bit shifting, feeds
  that into `std::mt19937_64`, and walks to the chosen array element
  using raw pointer arithmetic instead of `operator[]`.
- Stateless — it has no knowledge of quotes, authors, or SQLite. Its
  only input is a list of IDs and its only output is one of them.

### 4. Rust Database Manager & Safety Layer

- Location: `rust-db/`
- The **only** service permitted to open `quotes.db`. No other service
  reads or writes SQLite directly.
- Implements two RPCs:
  - `GetAllIds(Empty) -> IdList`
  - `GetQuoteById(QuoteRequest) -> QuoteResponse`
- All queries are parameterized (`rusqlite::params!`) — no string
  interpolation into SQL, ever — which is the "safety layer" half of
  its name.
- On first startup, creates the `quotes` table if missing and seeds it
  with ten dummy quotes if empty.

### 5. SQLite (`quotes.db`)

- Schema:
  ```sql
  CREATE TABLE quotes (
      id     INTEGER PRIMARY KEY AUTOINCREMENT,
      quote  TEXT NOT NULL,
      author TEXT NOT NULL
  );
  ```
- Lives at `rust-db/quotes.db`, created automatically by `rust-db` on
  first run.

## Data flow

```
Frontend          Go Gateway            Rust (QuoteDb)        C++ (Randomizer)
   │  GET /api/quote  │                        │                      │
   │ ────────────────▶│                        │                      │
   │                   │  GetAllIds()           │                      │
   │                   │ ──────────────────────▶│                      │
   │                   │◀── IdList [1,2,5,8,12] │                      │
   │                   │                        │                      │
   │                   │  SelectRandomId(IdList)                       │
   │                   │ ──────────────────────────────────────────────▶
   │                   │◀───────────────────── SelectedId { id: 8 } ───│
   │                   │                        │                      │
   │                   │  GetQuoteById(id=8)    │                      │
   │                   │ ──────────────────────▶│                      │
   │                   │◀── QuoteResponse       │                      │
   │◀── JSON quote ────│                        │                      │
```

Each arrow is a separate network round-trip (loopback gRPC calls in
this local setup), which is the entire joke: a query that SQLite could
answer in microseconds with `ORDER BY RANDOM() LIMIT 1` instead takes
three RPCs across two other processes.

## Why gRPC + Protobuf

- **Single source of truth for the contract**: `proto/quote.proto`
  defines every message and RPC once; Rust and C++ generate their
  server-side stubs from it, Go generates its client-side stubs from
  it. There's no hand-written serialization code anywhere.
- **Language-appropriate codegen**: `tonic-build` for Rust (invoked
  automatically via `build.rs`), `protoc` + the gRPC C++ plugin for
  C++ (invoked automatically via CMake's `add_custom_command`), and
  `protoc-gen-go`/`protoc-gen-go-grpc` for Go (invoked via
  `just proto-go`).

## Failure handling

The Go gateway treats any gRPC error from either backend as a
`502 Bad Gateway` to the frontend, with a JSON body of the form
`{"error": "..."}`. An empty ID list from Rust (e.g. a freshly wiped
database) is surfaced as `404 Not Found` instead, since that's a valid
state rather than a backend failure.

See [API reference](api.md) for the exact response shapes.

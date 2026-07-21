# API Reference

## HTTP: Go API Gateway

The only endpoint the frontend (or anything else) is meant to call.

### `GET /api/quote`

Runs the full three-step orchestration (Rust → C++ → Rust) and returns
one random quote.

**Request**

```
GET /api/quote HTTP/1.1
Host: localhost:8080
```

No parameters, no body.

**Success response** — `200 OK`

```json
{
  "quote": "Talk is cheap. Show me the code.",
  "author": "Linus Torvalds"
}
```

**Error responses**

| Status                   | Meaning                                          | Body                                                                                  |
| ------------------------ | ------------------------------------------------ | ------------------------------------------------------------------------------------- |
| `404 Not Found`          | Database has zero quotes                         | `{"error": "no quotes available"}`                                                    |
| `405 Method Not Allowed` | Called with a method other than `GET`            | `{"error": "method not allowed"}`                                                     |
| `502 Bad Gateway`        | The Rust or C++ service failed or is unreachable | `{"error": "failed to fetch quote ids"}` (or similar, depending on which step failed) |

CORS is wide open (`Access-Control-Allow-Origin: *`) since this is a
local demo app served from a different port than the API.

---

## gRPC: `QuoteDb` service (Rust)

Defined in `proto/quote.proto`. Listens on `localhost:50051`.

### `GetAllIds(Empty) -> IdList`

Returns every quote ID currently in `quotes.db`, ordered ascending.

```protobuf
message Empty {}

message IdList {
  repeated int64 ids = 1;
}
```

### `GetQuoteById(QuoteRequest) -> QuoteResponse`

Returns the quote text and author for a single ID.

```protobuf
message QuoteRequest {
  int64 id = 1;
}

message QuoteResponse {
  string quote = 1;
  string author = 2;
}
```

Returns a gRPC `NOT_FOUND` status if the ID doesn't exist.

---

## gRPC: `Randomizer` service (C++)

Defined in `proto/quote.proto`. Listens on `localhost:50052`.

### `SelectRandomId(IdList) -> SelectedId`

Given a non-empty list of IDs, returns exactly one of them, chosen via
`std::random_device` + `std::mt19937_64`.

```protobuf
message SelectedId {
  int64 id = 1;
}
```

Returns a gRPC `INVALID_ARGUMENT` status if `ids` is empty.

---

## Full proto file

See [`proto/quote.proto`](../proto/quote.proto) for the authoritative,
up-to-date definitions — this document mirrors it for convenience but
the `.proto` file is the source of truth.

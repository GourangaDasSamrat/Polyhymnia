# Polyhymnia

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Go](https://img.shields.io/badge/Go-00ADD8?style=flat&logo=go&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![C++](https://img.shields.io/badge/C%2B%2B-00599C?style=flat&logo=c%2B%2B&logoColor=white)

**The Ultimate Overengineered Random Quote Generator.**

Click a button. Get a quote. Behind the scenes, four services in four
languages talk to each other over gRPC to make that happen — because a
single `SELECT ... ORDER BY RANDOM()` would have been far too easy.

![App preview](https://i.postimg.cc/RCW1vbzF/Screenshot-2026-07-20-at-16-46-06-Polyhymnia-The-Ultimate-Overengineered-Quote-Generator.png)
---

## Name Origin

**The mythology:** In Greek mythology, Polyhymnia was the goddess of sacred poetry, melody, and words.

**The pun:** The name derives from Greek _Poly_ (many) + _Hymnia_ (words). But here's the twist—_Poly_ also represents **Polyglot** (Go, C++, Rust, JavaScript), the four languages used in this project. So Polyhymnia means: _a message (Quote) delivered through many languages_.

---

## What this actually is

A polyglot demo application built around one deliberately absurd
constraint: picking one random quote out of a small SQLite database is
split across **five hops** through **four languages**, connected by
**gRPC/Protobuf**.

```
┌────────────┐   HTTP GET    ┌──────────────┐
│  Frontend  │ ────────────▶ │  Go Gateway  │
│ (JS/HTML)  │ ◀──────────── │ (Orchestrator│
└────────────┘   JSON quote  └──────┬───────┘
                                     │
                 ┌───────────────────┼───────────────────┐
                 │ 1. GetAllIds       │ 3. GetQuoteById    │
                 ▼                    │                    ▼
          ┌─────────────┐             │             ┌─────────────┐
          │  Rust "db"  │◀────────────┘             │  Rust "db"  │
          │  service    │                            │  service    │
          └──────┬──────┘                            └─────────────┘
                 │ 2. SelectRandomId
                 ▼
          ┌─────────────┐
          │  C++ engine │
          │ (Randomizer)│
          └─────────────┘
```

## Tech stack

| Layer                           | Language                      | Responsibility                                                |
| ------------------------------- | ----------------------------- | ------------------------------------------------------------- |
| Frontend                        | Plain JavaScript / HTML / CSS | Single button, displays the quote                             |
| API Gateway / Orchestrator      | Go                            | HTTP entrypoint, drives the 3-step gRPC workflow              |
| Mathematical Engine             | C++                           | Picks one ID at random from a list, with unnecessary ceremony |
| Database Manager & Safety Layer | Rust                          | Owns `quotes.db`, the only service allowed to touch SQLite    |
| Database                        | SQLite                        | Stores `id`, `quote`, `author`                                |
| Inter-service transport         | gRPC + Protobuf               | Contract lives in `proto/quote.proto`                         |

## Data flow

1. User clicks **Get Quote** in the browser → `GET /api/quote` on the Go gateway.
2. Go → Rust: `GetAllIds` → every quote ID in the database.
3. Go → C++: `SelectRandomId(ids)` → one ID, chosen via `std::random_device`,
   a hand-rolled bit-mixing step, and raw pointer arithmetic.
4. Go → Rust: `GetQuoteById(id)` → the quote text and author.
5. Go → Frontend: `{ "quote": "...", "author": "..." }` as JSON.

## Project layout

```
Polyhymnia/
├── docker/                    # Dockerfiles for each service and compose
├── proto/                     # Shared .proto contract (quote.proto)
├── go-gateway/                # Go HTTP server + gRPC clients (orchestrator)
├── cpp-engine/                # C++ gRPC server (Randomizer service)
├── rust-db/                   # Rust gRPC server + SQLite (QuoteDb service)
├── frontend/                  # Static HTML/CSS/JS UI
├── docs/                      # Installation & reference docs
├── justfile                   # Build/run recipes for every service
├── docker-compose.yml         # Docker compose file (local builds)
├── docker-compose.images.yml  # Docker compose file (pre-built images)
└── .gitignore
```

## Quickstart

Get Polyhymnia running in minutes: **[Quickstart Guide](docs/quickstart.md)**.

See the detailed [Installation guide](docs/installation.md) for full setup instructions.

## Docs

- [Installation guide](docs/installation.md) — what to install on Linux and macOS
- [Architecture](docs/architecture.md) — service responsibilities and the gRPC contract in detail
- [API reference](docs/api.md) — the one HTTP endpoint, and the two gRPC services behind it
- 📖 **[System Design Series](https://dev.to/gouranga-das-khulna/series/42362)** — in-depth blog series on dev.to covering the project design and implementation
- [Troubleshooting](docs/troubleshooting.md) — common build/run errors and fixes

## Author

Name: Gouranga Das Samrat
GitHub: https://github.com/GourangaDasSamrat
Email: gouranga.samrat@gmail.com

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.

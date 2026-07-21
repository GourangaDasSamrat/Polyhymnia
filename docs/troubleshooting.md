# Troubleshooting

Common problems when building/running Polyhymnia on Linux or macOS, and
how to fix them.

## `just build-cpp` fails

### `Could not find a package configuration file provided by "gRPC"`

```
CMake Error at CMakeLists.txt:14 (find_package):
  Could not find a package configuration file provided by "gRPC" ...
```

This means CMake couldn't find `gRPCConfig.cmake`. This is expected on
many Linux distros — `apt`/`dnf` gRPC packages usually ship
`pkg-config` `.pc` files instead of CMake config packages.

`cpp-engine/CMakeLists.txt` already handles this: it tries
`find_package(gRPC CONFIG)` first, and falls back to `pkg-config` if
that's not found. If you still hit this error, it means the fallback
also failed — see the next entry.

<a id="grpc-not-found-on-linux"></a>

### `No package 'grpc++' found` (pkg-config fallback also fails)

```
-- Checking for module 'grpc++'
--   No package 'grpc++' found
```

This means neither CMake config packages nor `pkg-config` `.pc` files
for gRPC are on your system at all. Check what's actually installed:

```bash
pkg-config --list-all | grep -i grpc
dpkg -L libgrpc++-dev | grep -E '\.pc$|\.cmake$'   # Debian/Ubuntu
```

If nothing shows up, your distro's gRPC dev package is incomplete or
missing. Options, in order of preference:

1. **Reinstall the dev package** — on Debian/Ubuntu:
   ```bash
   sudo apt install --reinstall libgrpc++-dev protobuf-compiler-grpc
   ```
2. **Build gRPC from source** (only if step 1 doesn't help — this
   takes a while):
   ```bash
   git clone --recurse-submodules -b v1.63.0 --depth 1 \
       https://github.com/grpc/grpc
   cd grpc
   mkdir -p cmake/build && cd cmake/build
   cmake -DgRPC_INSTALL=ON -DgRPC_BUILD_TESTS=OFF \
         -DCMAKE_BUILD_TYPE=Release ../..
   make -j"$(nproc)"
   sudo make install
   ```
   This installs proper `gRPCConfig.cmake` files, so the `CONFIG`
   branch in `CMakeLists.txt` will succeed directly afterward.

### `protoc-gen-grpc` / `grpc_cpp_plugin` not found

```
find_program(GRPC_CPP_PLUGIN_EXE grpc_cpp_plugin REQUIRED)
```

The gRPC C++ codegen plugin isn't on your `PATH`. On Debian/Ubuntu it
comes from `protobuf-compiler-grpc`:

```bash
sudo apt install protobuf-compiler-grpc
which grpc_cpp_plugin
```

On macOS with Homebrew, `brew install grpc` installs it automatically.

---

## `just build-go` fails

### `cannot find package` / missing `quotepb` types

```
go-gateway/main.go:21:2: package polyhymnia/proto is not in std
```

This "is not in std" phrasing is Go's generic fallback message when it
can't resolve an import at all — it applies to two different root
causes, so check both:

1. **The Go gRPC stubs haven't been generated yet.** `go-gateway/proto/`
   only ships a placeholder (`generate.go`) documenting how to generate
   the real `*.pb.go` files — run:

   ```bash
   just proto-go
   ```

   This requires `protoc-gen-go` and `protoc-gen-go-grpc` to be
   installed and on your `PATH` — see
   [installation.md](installation.md#4-go-and-rust-protoc-plugins).

2. **The import path doesn't match the module layout.** `go-gateway/go.mod`
   declares `module polyhymnia`, rooted at the `go-gateway/` directory
   itself — so the `proto` subfolder is import path `polyhymnia/proto`,
   _not_ `polyhymnia/go-gateway/proto`. If you ever restructure the repo
   or copy files around, make sure `main.go`'s import, `proto/quote.proto`'s
   `go_package` option, and the actual directory nesting all agree with
   where `go.mod` lives.

### `go: command not found` after installing Go

Your shell doesn't have Go's `bin` directory on `PATH` yet. Re-source
your shell config or open a new terminal:

```bash
source ~/.bashrc   # or ~/.zshrc
```

---

## `just build-rust` fails

### Linker errors mentioning SQLite symbols

`rusqlite`'s `bundled` feature compiles SQLite's C source itself, which
needs a working C compiler:

```bash
# Debian/Ubuntu
sudo apt install build-essential

# macOS
xcode-select --install
```

### `error: failed to run custom build command for rust-db`

This is usually `tonic-build` failing to find `protoc`. Confirm it's
installed and on `PATH`:

```bash
protoc --version
```

---

## Everything builds, but the frontend shows a connection error

1. Confirm all three backend processes are actually running:
   ```bash
   lsof -i :50051   # rust-db
   lsof -i :50052   # cpp-engine
   lsof -i :8080    # go-gateway
   ```
2. If you're running services individually rather than via `just run`,
   make sure you started `rust-db` and `cpp-engine` **before**
   `go-gateway` — the gateway dials both on startup and will exit if
   either is unreachable.
3. Check the Go gateway's terminal output — it logs which downstream
   call failed (`GetAllIds`, `SelectRandomId`, or `GetQuoteById`).

## `quotes.db` seems empty / `GetAllIds` returns nothing

`rust-db` only seeds dummy data when the `quotes` table is empty on
startup. If you've since deleted rows manually, either:

- Delete `rust-db/quotes.db` entirely and restart `rust-db` (it will
  recreate and reseed it), or
- Insert rows manually with `sqlite3`:
  ```bash
  sqlite3 rust-db/quotes.db \
    "INSERT INTO quotes (quote, author) VALUES ('Your quote here', 'Someone');"
  ```

Still stuck? Open an issue in the repository with the exact command you
ran and the full error output.

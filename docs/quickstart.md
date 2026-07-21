# Quickstart Guide

Get Polyhymnia up and running in minutes.

## Clone and Setup

```bash
git clone https://github.com/gourangadassamrat/polyhymnia.git
cd polyhymnia
```

For detailed installation instructions (Linux & macOS): **[installation.md](installation.md)**.

---

## Method 1: Local Build (requires full toolchain)

Once your toolchain is set up:

```bash
just build   # generate gRPC code + compile all three backend services
just run     # build, then launch rust-db, cpp-engine, go-gateway, and the frontend together
```

Then open **http://localhost:5500** and click **Get Quote**.

---

## Method 2: Docker Compose (build locally)

Use Docker for an automatic setup (recommended for quick demos):

```bash
docker-compose build
docker-compose up
```

This runs the same services in containers and exposes the same ports.

---

## Method 3: Pre-built Docker Images (fastest)

Use pre-built images from Docker Hub with a dedicated compose file:

```bash
# Edit the file to replace <VERSION> with your desired tag
# (e.g., 1.0.0)
cat docker-compose.images.yml

# Then run
docker-compose -f docker-compose.images.yml up
```

Replace `<VERSION>` in `docker-compose.images.yml` with the desired release tag.
Check [available releases](https://hub.docker.com/r/gourangadassamrat/polyhymnia/tags)
on Docker Hub.

---

## Services and Ports

Once running, Polyhymnia exposes:

| Service               | Address                           |
| --------------------- | --------------------------------- |
| Frontend (static)     | `http://localhost:5500`           |
| Go API Gateway        | `http://localhost:8080/api/quote` |
| Rust QuoteDb (gRPC)   | `localhost:50051`                 |
| C++ Randomizer (gRPC) | `localhost:50052`                 |

---

## Troubleshooting

If something goes wrong, check [Troubleshooting](troubleshooting.md) or the
[Installation guide](installation.md) for more details.

//! Polyhymnia — Rust "Database Manager & Safety Layer".
//!
//! Owns `quotes.db` (SQLite) exclusively. No other service is allowed to
//! touch the database file directly; everything goes through the
//! `QuoteDb` gRPC service defined here, using parameterized queries only.

use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

pub mod quotepb {
    tonic::include_proto!("polyhymnia");
}

use quotepb::quote_db_server::{QuoteDb, QuoteDbServer};
use quotepb::{Empty, IdList, QuoteRequest, QuoteResponse};

const DB_PATH: &str = "quotes.db";
// Bind to IPv4 loopback explicitly. "[::1]" (IPv6-only) can silently
// mismatch clients that resolve "localhost" to 127.0.0.1 on systems
// causing "connection refused" even though the server is running.
const LISTEN_ADDR: &str = "127.0.0.1:50051";

/// Seed data inserted the first time `quotes.db` is created.
const SEED_QUOTES: &[(&str, &str)] = &[
    (
        "The only way to do great work is to love what you do.",
        "Steve Jobs",
    ),
    (
        "Life is what happens when you're busy making other plans.",
        "John Lennon",
    ),
    (
        "In the middle of difficulty lies opportunity.",
        "Albert Einstein",
    ),
    (
        "It does not matter how slowly you go as long as you do not stop.",
        "Confucius",
    ),
    (
        "Whether you think you can or you think you can't, you're right.",
        "Henry Ford",
    ),
    (
        "The journey of a thousand miles begins with a single step.",
        "Lao Tzu",
    ),
    (
        "Premature optimization is the root of all evil.",
        "Donald Knuth",
    ),
    ("Simplicity is the soul of efficiency.", "Austin Freeman"),
    ("Talk is cheap. Show me the code.", "Linus Torvalds"),
    (
        "Programs must be written for people to read.",
        "Harold Abelson",
    ),
];

struct QuoteDbService {
    conn: Arc<Mutex<Connection>>,
}

#[tonic::async_trait]
impl QuoteDb for QuoteDbService {
    async fn get_all_ids(&self, _request: Request<Empty>) -> Result<Response<IdList>, Status> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT id FROM quotes ORDER BY id")
            .map_err(|e| Status::internal(format!("prepare failed: {e}")))?;

        let ids: Result<Vec<i64>, rusqlite::Error> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| Status::internal(format!("query failed: {e}")))?
            .collect();

        let ids = ids.map_err(|e| Status::internal(format!("row read failed: {e}")))?;

        tracing::info!(count = ids.len(), "served GetAllIds");
        Ok(Response::new(IdList { ids }))
    }

    async fn get_quote_by_id(
        &self,
        request: Request<QuoteRequest>,
    ) -> Result<Response<QuoteResponse>, Status> {
        let id = request.into_inner().id;
        let conn = self.conn.lock().await;

        // Parameterized query — never string-interpolated — to keep this
        // service the "safety layer" the spec asks for.
        let result = conn.query_row(
            "SELECT quote, author FROM quotes WHERE id = ?1",
            params![id],
            |row| {
                let quote: String = row.get(0)?;
                let author: String = row.get(1)?;
                Ok((quote, author))
            },
        );

        match result {
            Ok((quote, author)) => {
                tracing::info!(id, "served GetQuoteById");
                Ok(Response::new(QuoteResponse { quote, author }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(Status::not_found(format!("no quote found for id {id}")))
            }
            Err(e) => Err(Status::internal(format!("query failed: {e}"))),
        }
    }
}

/// Opens (or creates) `quotes.db`, creates the schema if missing, and
/// seeds it with dummy data on first run.
fn init_db() -> rusqlite::Result<Connection> {
    let conn = Connection::open(DB_PATH)?;

    // Set busy timeout to 5 seconds to handle lock contention
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    // Enable WAL mode for better concurrency (PRAGMA returns results, so use query_row)
    let _: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;

    eprintln!("[rust-db] Database opened: {}", DB_PATH);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS quotes (
            id     INTEGER PRIMARY KEY AUTOINCREMENT,
            quote  TEXT NOT NULL,
            author TEXT NOT NULL
        )",
        [],
    )?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM quotes", [], |row| row.get(0))?;

    if count == 0 {
        eprintln!(
            "[rust-db] Seeding database with {} quotes",
            SEED_QUOTES.len()
        );
        let tx = conn.unchecked_transaction()?;
        for (quote, author) in SEED_QUOTES {
            tx.execute(
                "INSERT INTO quotes (quote, author) VALUES (?1, ?2)",
                params![quote, author],
            )?;
        }
        tx.commit()?;
    }

    eprintln!("[rust-db] Database initialized with {} quotes", count);
    Ok(conn)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    eprintln!("[rust-db] Starting database service...");

    let conn = init_db().map_err(|e| {
        eprintln!("[rust-db] [ERR] Failed to initialize database: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    eprintln!("[rust-db] [OK] Database initialized successfully");

    let service = QuoteDbService {
        conn: Arc::new(Mutex::new(conn)),
    };

    let addr = LISTEN_ADDR.parse().map_err(|e| {
        eprintln!("[rust-db] [ERR] Failed to parse address: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    eprintln!("[rust-db] [OK] Binding to {}", addr);
    tracing::info!(%addr, "rust-db (QuoteDb) service starting");

    Server::builder()
        .add_service(QuoteDbServer::new(service))
        .serve(addr)
        .await
        .map_err(|e| {
            eprintln!("[rust-db] [ERR] Server error: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    eprintln!("[rust-db] Server stopped normally");
    Ok(())
}

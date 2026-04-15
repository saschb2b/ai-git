use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Path::new(".aig").join("aig.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path)?;
        Ok(Self { conn })
    }

    pub fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS intents (
                id TEXT PRIMARY KEY,
                description TEXT,
                parent_id TEXT,
                created_at TEXT,
                closed_at TEXT,
                summary TEXT
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                intent_id TEXT,
                started_at TEXT,
                ended_at TEXT
            );

            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                intent_id TEXT,
                git_commit_sha TEXT,
                message TEXT,
                created_at TEXT
            );

            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                message TEXT,
                created_at TEXT
            );

            CREATE TABLE IF NOT EXISTS semantic_changes (
                id TEXT PRIMARY KEY,
                checkpoint_id TEXT,
                file_path TEXT,
                change_type TEXT,
                symbol_name TEXT,
                details TEXT
            );

            CREATE TABLE IF NOT EXISTS provenance (
                id TEXT PRIMARY KEY,
                checkpoint_id TEXT,
                file_path TEXT,
                start_line INTEGER,
                end_line INTEGER,
                origin TEXT,
                reviewed INTEGER DEFAULT 0,
                reviewed_at TEXT,
                reviewed_by TEXT
            );
            ",
        )?;
        Ok(())
    }
}

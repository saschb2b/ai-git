use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::db::Database;

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub intent_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

pub struct SessionManager;

impl SessionManager {
    fn generate_id(seed: &str) -> String {
        let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let input = format!("{seed}-{now}");
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(&hasher.finalize()[..16])
    }

    pub fn start_session(db: &Database, intent: &str) -> Result<String> {
        let id = Self::generate_id(intent);
        let now = Utc::now().to_rfc3339();
        db.conn.execute(
            "INSERT INTO sessions (id, intent_id, started_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, intent, now],
        )?;
        Ok(id)
    }

    pub fn end_session(db: &Database, session_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        db.conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            rusqlite::params![now, session_id],
        )?;
        Ok(())
    }

    pub fn get_active_session(db: &Database) -> Result<Option<Session>> {
        let mut stmt = db.conn.prepare(
            "SELECT id, intent_id, started_at, ended_at FROM sessions WHERE ended_at IS NULL ORDER BY started_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                intent_id: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}

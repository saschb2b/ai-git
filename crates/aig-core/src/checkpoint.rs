use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::db::Database;

pub struct CheckpointManager;

impl CheckpointManager {
    fn generate_id(seed: &str) -> String {
        let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        let input = format!("checkpoint-{seed}-{now}");
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(&hasher.finalize()[..16])
    }

    pub fn create_checkpoint(
        db: &Database,
        session_id: &str,
        message: &str,
        git_sha: &str,
    ) -> Result<String> {
        let id = Self::generate_id(message);
        let now = Utc::now().to_rfc3339();

        // Look up intent_id from the session
        let intent_id: String = db.conn.query_row(
            "SELECT intent_id FROM sessions WHERE id = ?1",
            rusqlite::params![session_id],
            |row| row.get(0),
        )?;

        db.conn.execute(
            "INSERT INTO checkpoints (id, session_id, intent_id, git_commit_sha, message, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, session_id, intent_id, git_sha, message, now],
        )?;
        Ok(id)
    }
}

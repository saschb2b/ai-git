use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::db::Database;

#[derive(Debug)]
pub struct Intent {
    pub id: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub created_at: String,
    pub closed_at: Option<String>,
    pub summary: Option<String>,
}

fn generate_id(seed: &str) -> String {
    let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let input = format!("intent-{seed}-{now}");
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(&hasher.finalize()[..16])
}

pub fn create_intent(db: &Database, description: &str) -> Result<String> {
    let id = generate_id(description);
    let now = Utc::now().to_rfc3339();
    db.conn.execute(
        "INSERT INTO intents (id, description, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, description, now],
    )?;
    Ok(id)
}

pub fn get_intent(db: &Database, id: &str) -> Result<Intent> {
    let intent = db.conn.query_row(
        "SELECT id, description, parent_id, created_at, closed_at, summary FROM intents WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Intent {
                id: row.get(0)?,
                description: row.get(1)?,
                parent_id: row.get(2)?,
                created_at: row.get(3)?,
                closed_at: row.get(4)?,
                summary: row.get(5)?,
            })
        },
    )?;
    Ok(intent)
}

pub fn list_intents(db: &Database) -> Result<Vec<Intent>> {
    let mut stmt = db.conn.prepare(
        "SELECT id, description, parent_id, created_at, closed_at, summary FROM intents ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Intent {
            id: row.get(0)?,
            description: row.get(1)?,
            parent_id: row.get(2)?,
            created_at: row.get(3)?,
            closed_at: row.get(4)?,
            summary: row.get(5)?,
        })
    })?;
    let mut intents = Vec::new();
    for row in rows {
        intents.push(row?);
    }
    Ok(intents)
}

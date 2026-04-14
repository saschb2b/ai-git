use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::db::Database;

const NOTES_REF: &str = "refs/notes/aig";

// ── Serialization structs ────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct AigNote {
    version: u32,
    intent: NoteIntent,
    session: Option<NoteSession>,
    checkpoint: NoteCheckpoint,
    semantic_changes: Vec<NoteSemanticChange>,
    conversations: Vec<NoteConversation>,
}

#[derive(Serialize, Deserialize)]
struct NoteIntent {
    id: String,
    description: String,
    parent_id: Option<String>,
    created_at: String,
    closed_at: Option<String>,
    summary: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct NoteSession {
    id: String,
    intent_id: String,
    started_at: String,
    ended_at: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct NoteCheckpoint {
    id: String,
    session_id: Option<String>,
    intent_id: String,
    message: String,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct NoteSemanticChange {
    file_path: String,
    change_type: String,
    symbol_name: String,
    details: String,
}

#[derive(Serialize, Deserialize)]
struct NoteConversation {
    message: String,
    created_at: String,
}

// ── Push local DB → git notes ────────────────────────────────────────

pub fn push_notes(db: &Database, repo: &git2::Repository) -> Result<usize> {
    let mut stmt = db.conn.prepare(
        "SELECT id, session_id, intent_id, git_commit_sha, message, created_at FROM checkpoints",
    )?;

    struct CpRow {
        id: String,
        session_id: Option<String>,
        intent_id: String,
        git_commit_sha: String,
        message: String,
        created_at: String,
    }

    let rows: Vec<CpRow> = stmt
        .query_map([], |row| {
            Ok(CpRow {
                id: row.get(0)?,
                session_id: row.get(1)?,
                intent_id: row.get(2)?,
                git_commit_sha: row.get(3)?,
                message: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let total = rows.len();
    let sig = repo.signature()?;

    for (i, cp) in rows.iter().enumerate() {
        // Intent
        let intent: NoteIntent = db.conn.query_row(
            "SELECT id, description, parent_id, created_at, closed_at, summary FROM intents WHERE id = ?",
            [&cp.intent_id],
            |row| {
                Ok(NoteIntent {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    parent_id: row.get(2)?,
                    created_at: row.get(3)?,
                    closed_at: row.get(4)?,
                    summary: row.get(5)?,
                })
            },
        )?;

        // Session (optional)
        let session: Option<NoteSession> = match &cp.session_id {
            Some(sid) => {
                let s = db.conn.query_row(
                    "SELECT id, intent_id, started_at, ended_at FROM sessions WHERE id = ?",
                    [sid],
                    |row| {
                        Ok(NoteSession {
                            id: row.get(0)?,
                            intent_id: row.get(1)?,
                            started_at: row.get(2)?,
                            ended_at: row.get(3)?,
                        })
                    },
                )?;
                Some(s)
            }
            None => None,
        };

        // Semantic changes
        let mut sc_stmt = db.conn.prepare(
            "SELECT file_path, change_type, symbol_name, details FROM semantic_changes WHERE checkpoint_id = ?",
        )?;
        let semantic_changes: Vec<NoteSemanticChange> = sc_stmt
            .query_map([&cp.id], |row| {
                Ok(NoteSemanticChange {
                    file_path: row.get(0)?,
                    change_type: row.get(1)?,
                    symbol_name: row.get(2)?,
                    details: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Conversations for this intent (across all sessions)
        let mut conv_stmt = db.conn.prepare(
            "SELECT c.message, c.created_at FROM conversations c JOIN sessions s ON c.session_id = s.id WHERE s.intent_id = ?",
        )?;
        let conversations: Vec<NoteConversation> = conv_stmt
            .query_map([&cp.intent_id], |row| {
                Ok(NoteConversation {
                    message: row.get(0)?,
                    created_at: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let note = AigNote {
            version: 1,
            intent,
            session,
            checkpoint: NoteCheckpoint {
                id: cp.id.clone(),
                session_id: cp.session_id.clone(),
                intent_id: cp.intent_id.clone(),
                message: cp.message.clone(),
                created_at: cp.created_at.clone(),
            },
            semantic_changes,
            conversations,
        };

        let json_string = serde_json::to_string_pretty(&note)?;

        let oid = git2::Oid::from_str(&cp.git_commit_sha)?;

        println!("Writing note {}/{} for checkpoint: {}", i + 1, total, cp.id);

        repo.note(&sig, &sig, Some(NOTES_REF), oid, &json_string, true)?;
    }

    Ok(total)
}

// ── Pull git notes → local DB ────────────────────────────────────────

pub fn pull_notes(db: &Database, repo: &git2::Repository) -> Result<usize> {
    let notes_iter = match repo.notes(Some(NOTES_REF)) {
        Ok(iter) => iter,
        Err(e) => {
            // If the notes ref doesn't exist yet, that's fine — nothing to pull.
            if e.code() == git2::ErrorCode::NotFound {
                return Ok(0);
            }
            return Err(e.into());
        }
    };

    let mut count = 0usize;

    for item in notes_iter {
        let (note_oid, annotated_oid) = match item {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("Warning: failed to read note entry: {e}");
                continue;
            }
        };

        // Read note blob
        let blob = repo
            .find_blob(note_oid)
            .context("failed to find note blob")?;
        let content = std::str::from_utf8(blob.content());
        let json_str = match content {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: note {note_oid} is not valid UTF-8: {e}");
                continue;
            }
        };

        let note: AigNote = match serde_json::from_str(json_str) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Warning: note {note_oid} has invalid JSON, skipping: {e}");
                continue;
            }
        };

        let commit_sha = annotated_oid.to_string();

        // Upsert intent
        db.conn.execute(
            "INSERT OR IGNORE INTO intents (id, description, parent_id, created_at, closed_at, summary) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                note.intent.id,
                note.intent.description,
                note.intent.parent_id,
                note.intent.created_at,
                note.intent.closed_at,
                note.intent.summary,
            ],
        )?;

        // Upsert session
        if let Some(ref sess) = note.session {
            db.conn.execute(
                "INSERT OR IGNORE INTO sessions (id, intent_id, started_at, ended_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![sess.id, sess.intent_id, sess.started_at, sess.ended_at],
            )?;
        }

        // Upsert checkpoint
        db.conn.execute(
            "INSERT OR IGNORE INTO checkpoints (id, session_id, intent_id, git_commit_sha, message, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                note.checkpoint.id,
                note.checkpoint.session_id,
                note.checkpoint.intent_id,
                commit_sha,
                note.checkpoint.message,
                note.checkpoint.created_at,
            ],
        )?;

        // Upsert semantic changes
        for sc in &note.semantic_changes {
            // Generate a deterministic ID from checkpoint_id + file_path + symbol_name
            let sc_id = deterministic_id(&[&note.checkpoint.id, &sc.file_path, &sc.symbol_name]);
            db.conn.execute(
                "INSERT OR IGNORE INTO semantic_changes (id, checkpoint_id, file_path, change_type, symbol_name, details) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    sc_id,
                    note.checkpoint.id,
                    sc.file_path,
                    sc.change_type,
                    sc.symbol_name,
                    sc.details,
                ],
            )?;
        }

        // Upsert conversations
        // We need a session_id to insert conversations. Use the session from the note if present.
        if let Some(ref sess) = note.session {
            for conv in &note.conversations {
                let conv_id = deterministic_id(&[&sess.id, &conv.message, &conv.created_at]);
                db.conn.execute(
                    "INSERT OR IGNORE INTO conversations (id, session_id, message, created_at) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![conv_id, sess.id, conv.message, conv.created_at],
                )?;
            }
        }

        count += 1;
    }

    Ok(count)
}

// ── Remote transport (shell out to git CLI) ──────────────────────────

pub fn push_to_remote(repo_path: &str, remote_name: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["push", remote_name, NOTES_REF])
        .current_dir(repo_path)
        .output()
        .context("failed to run git push")?;

    if !output.stdout.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        anyhow::bail!(
            "git push {} {} failed with exit code {:?}",
            remote_name,
            NOTES_REF,
            output.status.code()
        );
    }

    Ok(())
}

pub fn pull_from_remote(repo_path: &str, remote_name: &str) -> Result<()> {
    let refspec = format!("{NOTES_REF}:{NOTES_REF}");
    let output = std::process::Command::new("git")
        .args(["fetch", remote_name, &refspec])
        .current_dir(repo_path)
        .output()
        .context("failed to run git fetch")?;

    if !output.stdout.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        anyhow::bail!(
            "git fetch {} {} failed with exit code {:?}",
            remote_name,
            refspec,
            output.status.code()
        );
    }

    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Produce a deterministic hex ID by hashing the given parts.
fn deterministic_id(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update(b"\0");
    }
    let result = hasher.finalize();
    hex::encode(&result[..16]) // 32-char hex string
}

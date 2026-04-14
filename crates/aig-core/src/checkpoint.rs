use anyhow::Result;
use chrono::Utc;
use git2::Repository;
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
        repo: &Repository,
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

        record_semantic_changes(db, &id, repo, git_sha)?;

        Ok(id)
    }
}

fn record_semantic_changes(
    db: &Database,
    checkpoint_id: &str,
    repo: &Repository,
    git_sha: &str,
) -> Result<()> {
    let oid = git2::Oid::from_str(git_sha)?;
    let commit = repo.find_commit(oid)?;
    let commit_tree = commit.tree()?;

    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)?;

    for delta in diff.deltas() {
        let file_path = match delta.new_file().path() {
            Some(p) => p.to_string_lossy().to_string(),
            None => continue,
        };

        let lang = aig_treesitter::detect_language(&file_path);
        if lang == aig_treesitter::Language::Unknown {
            continue;
        }

        let new_content = match commit_tree
            .get_path(std::path::Path::new(&file_path))
            .ok()
            .and_then(|entry| repo.find_blob(entry.id()).ok())
        {
            Some(blob) => String::from_utf8_lossy(blob.content()).to_string(),
            None => String::new(),
        };

        let old_content = match parent_tree
            .as_ref()
            .and_then(|t| t.get_path(std::path::Path::new(&file_path)).ok())
            .and_then(|entry| repo.find_blob(entry.id()).ok())
        {
            Some(blob) => String::from_utf8_lossy(blob.content()).to_string(),
            None => String::new(),
        };

        let changes = match aig_treesitter::semantic_diff(&old_content, &new_content, lang) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for change in &changes {
            let id = CheckpointManager::generate_id(&format!(
                "{}-{}-{}",
                checkpoint_id, file_path, change.symbol_name
            ));

            db.conn.execute(
                "INSERT INTO semantic_changes (id, checkpoint_id, file_path, change_type, symbol_name, details) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    id,
                    checkpoint_id,
                    file_path,
                    change.change_type,
                    change.symbol_name,
                    change.details,
                ],
            )?;
        }
    }

    Ok(())
}

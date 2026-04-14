use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::db::Database;

const NOTES_REF: &str = "refs/notes/aig";

pub struct RepairResult {
    pub repaired: usize,
    pub orphaned: usize,
    pub ok: usize,
}

/// After a rebase, commit SHAs change but aig notes are still attached to old SHAs.
/// This function finds orphaned notes and re-attaches them to matching new commits.
///
/// The matching strategy:
/// 1. Walk all notes in refs/notes/aig
/// 2. For each note, check if the annotated commit still exists in the repo
/// 3. If not (orphaned), parse the note's JSON to get the checkpoint message
/// 4. Walk the current git log looking for a commit with the same message
/// 5. If found, re-attach the note to the new commit SHA and update the DB
pub fn repair_notes(db: &Database, repo: &git2::Repository) -> Result<RepairResult> {
    // Step 1: Build a lookup map of current commits by first-line message -> OID
    let mut commit_map: HashMap<String, git2::Oid> = HashMap::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head().ok(); // ok to fail if HEAD doesn't exist
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL)?;

    let mut walked = 0usize;
    for oid_result in revwalk {
        if walked >= 10_000 {
            break;
        }
        let oid = match oid_result {
            Ok(o) => o,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Some(summary) = commit.summary() {
            // Only keep the first occurrence (most recent commit wins due to topo order)
            commit_map.entry(summary.to_string()).or_insert(oid);
        }
        walked += 1;
    }

    // Step 2: Iterate all notes
    let notes_iter = match repo.notes(Some(NOTES_REF)) {
        Ok(iter) => iter,
        Err(e) => {
            if e.code() == git2::ErrorCode::NotFound {
                return Ok(RepairResult {
                    repaired: 0,
                    orphaned: 0,
                    ok: 0,
                });
            }
            return Err(e.into());
        }
    };

    // Collect notes first to avoid borrow issues
    let notes: Vec<(git2::Oid, git2::Oid)> = notes_iter.filter_map(|item| item.ok()).collect();

    let sig = repo.signature()?;
    let mut result = RepairResult {
        repaired: 0,
        orphaned: 0,
        ok: 0,
    };

    for (note_oid, annotated_oid) in &notes {
        // Step 3a: Check if the annotated commit still exists
        if repo.find_commit(*annotated_oid).is_ok() {
            result.ok += 1;
            continue;
        }

        // Step 3b: Orphaned note — read it and try to match
        let blob = repo
            .find_blob(*note_oid)
            .context("failed to find note blob")?;
        let json_str = match std::str::from_utf8(blob.content()) {
            Ok(s) => s,
            Err(_) => {
                result.orphaned += 1;
                continue;
            }
        };

        let note_value: serde_json::Value = match serde_json::from_str(json_str) {
            Ok(v) => v,
            Err(_) => {
                result.orphaned += 1;
                continue;
            }
        };

        // Extract the checkpoint message from the note JSON
        let checkpoint_message = match note_value
            .get("checkpoint")
            .and_then(|cp| cp.get("message"))
            .and_then(|m| m.as_str())
        {
            Some(msg) => msg.to_string(),
            None => {
                result.orphaned += 1;
                continue;
            }
        };

        // Step 4: Look up by message in the commit map
        if let Some(&new_oid) = commit_map.get(&checkpoint_message) {
            let old_sha = annotated_oid.to_string();
            let new_sha = new_oid.to_string();

            // Write the note content onto the new commit
            repo.note(
                &sig,
                &sig,
                Some(NOTES_REF),
                new_oid,
                json_str,
                true, // force overwrite if a note already exists
            )?;

            // Update the checkpoints table in the DB
            db.conn.execute(
                "UPDATE checkpoints SET git_commit_sha = ?1 WHERE git_commit_sha = ?2",
                rusqlite::params![new_sha, old_sha],
            )?;

            result.repaired += 1;
        } else {
            result.orphaned += 1;
        }
    }

    Ok(result)
}

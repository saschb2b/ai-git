use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::db::Database;
use crate::git_interop::{self, CommitInfo};
use crate::intent;

// ---------------------------------------------------------------------------
// IPC client for optional LLM-powered intent inference
// ---------------------------------------------------------------------------

/// Context for the `explain_line` IPC command.
pub struct ExplainLineContext {
    pub file_path: String,
    pub line: usize,
    pub intent_description: String,
    pub checkpoint_message: String,
    pub conversation_notes: Vec<String>,
    pub semantic_changes: Vec<String>,
    pub line_content: String,
}

/// A lightweight IPC client that communicates with a TypeScript child process
/// over NDJSON (newline-delimited JSON) on stdin/stdout.
pub struct IpcClient {
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
}

impl IpcClient {
    /// Try to spawn the TypeScript LLM helper process.
    ///
    /// Returns `None` if the package is not installed, Node is not available,
    /// or the process fails to start for any reason.
    pub fn try_connect(repo_root: &str) -> Option<IpcClient> {
        let script = Path::new(repo_root)
            .join("node_modules")
            .join("@aig")
            .join("llm")
            .join("dist")
            .join("index.js");

        if !script.exists() {
            return None;
        }

        let mut child = Command::new("node")
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let stdout = child.stdout.take()?;
        let reader = BufReader::new(stdout);

        Some(IpcClient { child, reader })
    }

    /// Send an `infer_intent` request and read the response.
    ///
    /// Returns `(intent, summary)` on success, or an error if the child
    /// process misbehaves.
    pub fn infer_intent(
        &mut self,
        messages: &[String],
        diff_stats: &[String],
    ) -> Result<(String, String)> {
        let request = serde_json::json!({
            "command": "infer_intent",
            "params": {
                "commit_messages": messages,
                "diff_stats": diff_stats,
            }
        });

        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("IPC stdin unavailable"))?;

        let mut line = serde_json::to_string(&request)?;
        line.push('\n');
        stdin.write_all(line.as_bytes())?;
        stdin.flush()?;

        let mut response_line = String::new();
        self.reader.read_line(&mut response_line)?;

        if response_line.is_empty() {
            anyhow::bail!("IPC process returned empty response");
        }

        let resp: serde_json::Value = serde_json::from_str(&response_line)?;

        let result = resp
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("IPC response missing 'result' field"))?;

        let intent = result
            .get("intent")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let summary = result
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok((intent, summary))
    }

    /// Send an `explain_line` request and read the response.
    ///
    /// Returns a natural-language explanation of why a line exists.
    pub fn explain_line(&mut self, ctx: &ExplainLineContext) -> Result<String> {
        let request = serde_json::json!({
            "command": "explain_line",
            "params": {
                "file_path": ctx.file_path,
                "line": ctx.line,
                "intent_description": ctx.intent_description,
                "checkpoint_message": ctx.checkpoint_message,
                "conversation_notes": ctx.conversation_notes,
                "semantic_changes": ctx.semantic_changes,
                "line_content": ctx.line_content,
            }
        });

        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("IPC stdin unavailable"))?;

        let mut line_buf = serde_json::to_string(&request)?;
        line_buf.push('\n');
        stdin.write_all(line_buf.as_bytes())?;
        stdin.flush()?;

        let mut response_line = String::new();
        self.reader.read_line(&mut response_line)?;

        if response_line.is_empty() {
            anyhow::bail!("IPC process returned empty response");
        }

        let resp: serde_json::Value = serde_json::from_str(&response_line)?;

        if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
            anyhow::bail!("LLM error: {err}");
        }

        let result = resp
            .get("result")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(result)
    }

    /// Send a `generate_summary` request and read the response.
    pub fn generate_summary(&mut self, changes: &[String]) -> Result<String> {
        let request = serde_json::json!({
            "command": "generate_summary",
            "params": {
                "changes": changes,
            }
        });

        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("IPC stdin unavailable"))?;

        let mut line = serde_json::to_string(&request)?;
        line.push('\n');
        stdin.write_all(line.as_bytes())?;
        stdin.flush()?;

        let mut response_line = String::new();
        self.reader.read_line(&mut response_line)?;

        if response_line.is_empty() {
            anyhow::bail!("IPC process returned empty response");
        }

        let resp: serde_json::Value = serde_json::from_str(&response_line)?;

        if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
            anyhow::bail!("LLM error: {err}");
        }

        let result = resp
            .get("result")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(result)
    }
}

impl Drop for IpcClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---------------------------------------------------------------------------
// Diff stats helper
// ---------------------------------------------------------------------------

/// Return per-file diff stats for a commit, e.g. `["src/auth.py: +42 -10"]`.
///
/// Diffs against the first parent (or the empty tree for root commits).
pub fn get_commit_diff_stats(
    repo: &git2::Repository,
    commit: &git2::Commit,
) -> Result<Vec<String>> {
    let tree = commit.tree()?;

    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    let stats_list = diff.stats()?;
    // Unfortunately git2's DiffStats only gives aggregate numbers, so we
    // iterate over deltas and compute per-file stats via the patches.
    let mut result = Vec::new();

    let num_deltas = diff.deltas().len();
    for idx in 0..num_deltas {
        let delta = diff.get_delta(idx);
        if delta.is_none() {
            continue;
        }
        let delta = delta.unwrap();

        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<unknown>".to_string());

        // Use the patch to count insertions / deletions for this file.
        if let Ok(Some(patch)) = git2::Patch::from_diff(&diff, idx) {
            let (_, additions, deletions) = patch.line_stats().unwrap_or((0, 0, 0));
            result.push(format!("{}: +{} -{}", path, additions, deletions));
        }
    }

    // If we couldn't get patches, fall back to aggregate stats only
    if result.is_empty() && stats_list.files_changed() > 0 {
        result.push(format!(
            "(aggregate): +{} -{}",
            stats_list.insertions(),
            stats_list.deletions()
        ));
    }

    Ok(result)
}

/// A cluster of related commits that likely represent a single intent.
#[derive(Debug)]
pub struct CommitCluster {
    pub commits: Vec<CommitInfo>,
    pub inferred_intent: String,
    pub summary: String,
}

/// Maximum gap (in seconds) between consecutive commits by the same author
/// to be considered part of the same cluster.
const CLUSTER_WINDOW_SECS: i64 = 2 * 60 * 60; // 2 hours

/// Import an existing git repository's history into aig.
///
/// This is the non-LLM version (MVP) — it clusters commits by heuristics
/// and generates intent descriptions from commit messages.
pub fn import_git_history(repo_path: &str) -> Result<()> {
    println!("Importing git history...");

    // 1. Open the git repo
    let repo = git_interop::open_repo(repo_path)?;

    // 2. Get the full log (up to 10 000 commits)
    let commits = git_interop::get_log(&repo, 10_000)?;
    println!("Found {} commits", commits.len());

    if commits.is_empty() {
        println!("No commits to import.");
        return Ok(());
    }

    // 3. Cluster commits using heuristics
    let clusters = cluster_commits(commits);
    println!("Clustered into {} intents", clusters.len());

    // 4. Try to connect to the optional LLM IPC server
    let mut ipc_client = IpcClient::try_connect(repo_path);
    if ipc_client.is_some() {
        println!("Using LLM for intent inference...");
    } else {
        println!("Using heuristic intent inference (install @aig/llm for better results)");
    }

    // 5. Initialize .aig/ database
    let db = Database::new()?;
    db.init_schema()?;

    // 6. For each cluster, create an intent and link commits as checkpoints
    let total_clusters = clusters.len();
    let mut new_intents: usize = 0;
    let mut new_commits: usize = 0;
    let mut skipped_commits: usize = 0;

    for (i, cluster) in clusters.iter().enumerate() {
        // Check which commits in this cluster are already imported
        let mut new_in_cluster = Vec::new();
        for commit in &cluster.commits {
            let exists: bool = db.conn.query_row(
                "SELECT COUNT(*) > 0 FROM checkpoints WHERE git_commit_sha = ?1",
                rusqlite::params![commit.sha],
                |row| row.get(0),
            )?;
            if exists {
                skipped_commits += 1;
            } else {
                new_in_cluster.push(commit);
            }
        }

        // If all commits in this cluster are already imported, skip the intent
        if new_in_cluster.is_empty() {
            continue;
        }

        // Determine intent + summary: LLM path or heuristic fallback
        let (final_intent, final_summary) = if let Some(ref mut client) = ipc_client {
            // Gather commit messages and diff stats for the LLM
            let messages: Vec<String> = cluster.commits.iter().map(|c| c.message.clone()).collect();

            let diff_stats: Vec<String> = cluster
                .commits
                .iter()
                .flat_map(|c| {
                    // Look up the actual git2 commit to compute diff stats
                    match repo.find_commit(
                        git2::Oid::from_str(&c.sha).unwrap_or_else(|_| git2::Oid::zero()),
                    ) {
                        Ok(git_commit) => {
                            get_commit_diff_stats(&repo, &git_commit).unwrap_or_default()
                        }
                        Err(_) => Vec::new(),
                    }
                })
                .collect();

            match client.infer_intent(&messages, &diff_stats) {
                Ok((intent, summary)) => (intent, summary),
                Err(e) => {
                    eprintln!(
                        "Warning: LLM inference failed for cluster {}, falling back to heuristics: {}",
                        i + 1,
                        e
                    );
                    (cluster.inferred_intent.clone(), cluster.summary.clone())
                }
            }
        } else {
            (cluster.inferred_intent.clone(), cluster.summary.clone())
        };

        println!(
            "Importing intent {}/{}: \"{}\"",
            i + 1,
            total_clusters,
            truncate_for_display(&final_intent, 60)
        );

        let intent_id = intent::create_intent(&db, &final_intent)?;

        // Optionally store the summary on the intent
        if !final_summary.is_empty() {
            db.conn.execute(
                "UPDATE intents SET summary = ?1 WHERE id = ?2",
                rusqlite::params![final_summary, intent_id],
            )?;
        }

        // Insert a checkpoint for each new commit in the cluster
        for commit in &new_in_cluster {
            let cp_id = generate_checkpoint_id(&commit.sha);
            let created_at = chrono::DateTime::from_timestamp(commit.timestamp, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339());

            db.conn.execute(
                "INSERT INTO checkpoints (id, session_id, intent_id, git_commit_sha, message, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    cp_id,
                    Option::<String>::None, // no session for imported commits
                    intent_id,
                    commit.sha,
                    commit.message,
                    created_at,
                ],
            )?;
        }

        // Imported intents represent completed historical work — close them
        let last_commit_time = new_in_cluster
            .last()
            .and_then(|c| chrono::DateTime::from_timestamp(c.timestamp, 0))
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        db.conn.execute(
            "UPDATE intents SET closed_at = ?1 WHERE id = ?2",
            rusqlite::params![last_commit_time, intent_id],
        )?;

        new_intents += 1;
        new_commits += new_in_cluster.len();
    }

    // 7. Print summary
    if new_intents == 0 && skipped_commits > 0 {
        println!("All commits already imported. Nothing to do.");
    } else if skipped_commits > 0 {
        println!(
            "Import complete: {} new intents created from {} commits ({} already imported, skipped)",
            new_intents, new_commits, skipped_commits
        );
    } else {
        println!(
            "Import complete: {} new intents created from {} commits",
            new_intents, new_commits
        );
    }

    Ok(())
}

/// Cluster commits by author + time proximity.
///
/// Commits are sorted oldest-first, then consecutive commits by the same
/// author within a 2-hour window are grouped together.
pub fn cluster_commits(mut commits: Vec<CommitInfo>) -> Vec<CommitCluster> {
    if commits.is_empty() {
        return Vec::new();
    }

    // get_log returns newest-first (git2 TIME sorting). Reverse to oldest-first.
    commits.reverse();

    let mut clusters: Vec<CommitCluster> = Vec::new();
    let mut current_group: Vec<CommitInfo> = Vec::new();

    for commit in commits {
        let should_start_new = if let Some(last) = current_group.last() {
            // Different author or time gap exceeds the window
            last.author != commit.author
                || (commit.timestamp - last.timestamp).abs() > CLUSTER_WINDOW_SECS
        } else {
            false
        };

        if should_start_new {
            // Flush the current group as a cluster
            clusters.push(build_cluster(std::mem::take(&mut current_group)));
        }

        current_group.push(commit);
    }

    // Flush the last group
    if !current_group.is_empty() {
        clusters.push(build_cluster(current_group));
    }

    clusters
}

/// Build a `CommitCluster` from a non-empty group of commits.
fn build_cluster(commits: Vec<CommitInfo>) -> CommitCluster {
    let (inferred_intent, summary) = infer_intent_description(&commits);
    CommitCluster {
        commits,
        inferred_intent,
        summary,
    }
}

/// Derive an intent description and summary from the commit messages in a cluster.
fn infer_intent_description(commits: &[CommitInfo]) -> (String, String) {
    if commits.is_empty() {
        return (String::new(), String::new());
    }

    let first_message = first_line(&commits[0].message);

    if commits.len() == 1 {
        (first_message.to_string(), first_message.to_string())
    } else {
        let intent = first_message.to_string();
        let summary = format!(
            "{} ({} commits: {})",
            first_message,
            commits.len(),
            commits
                .iter()
                .map(|c| first_line(&c.message).to_string())
                .collect::<Vec<_>>()
                .join("; ")
        );
        (intent, summary)
    }
}

/// Get the first line of a commit message, trimmed.
fn first_line(msg: &str) -> &str {
    msg.lines().next().unwrap_or("").trim()
}

/// Truncate a string for display purposes.
fn truncate_for_display(s: &str, max_len: usize) -> String {
    let line = first_line(s);
    if line.len() <= max_len {
        line.to_string()
    } else {
        format!("{}...", &line[..max_len.saturating_sub(3)])
    }
}

/// Generate a deterministic checkpoint ID from a commit SHA.
fn generate_checkpoint_id(sha: &str) -> String {
    let now = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let input = format!("import-checkpoint-{sha}-{now}");
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(&hasher.finalize()[..16])
}

/// Get the list of file paths changed in a commit by diffing against its parent.
///
/// For merge commits, diffs against the first parent. For root commits (no
/// parent), all files in the tree are considered changed.
pub fn get_commit_files(repo: &git2::Repository, commit: &git2::Commit) -> Result<Vec<String>> {
    let tree = commit.tree()?;

    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    let mut files = Vec::new();
    for delta in diff.deltas() {
        if let Some(path) = delta.new_file().path() {
            files.push(path.to_string_lossy().into_owned());
        } else if let Some(path) = delta.old_file().path() {
            files.push(path.to_string_lossy().into_owned());
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_single_author_within_window() {
        let commits = vec![
            CommitInfo {
                sha: "aaa".into(),
                message: "first".into(),
                author: "Alice".into(),
                timestamp: 1000,
            },
            CommitInfo {
                sha: "bbb".into(),
                message: "second".into(),
                author: "Alice".into(),
                timestamp: 2000,
            },
        ];
        // Input is newest-first (like get_log), so reverse happens in cluster_commits
        let clusters = cluster_commits(commits);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].commits.len(), 2);
    }

    #[test]
    fn test_cluster_different_authors() {
        let commits = vec![
            CommitInfo {
                sha: "bbb".into(),
                message: "Bob's commit".into(),
                author: "Bob".into(),
                timestamp: 2000,
            },
            CommitInfo {
                sha: "aaa".into(),
                message: "Alice's commit".into(),
                author: "Alice".into(),
                timestamp: 1000,
            },
        ];
        let clusters = cluster_commits(commits);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn test_cluster_time_gap() {
        let base = 1_000_000;
        let commits = vec![
            CommitInfo {
                sha: "bbb".into(),
                message: "later".into(),
                author: "Alice".into(),
                timestamp: base + 3 * 60 * 60, // 3 hours later
            },
            CommitInfo {
                sha: "aaa".into(),
                message: "early".into(),
                author: "Alice".into(),
                timestamp: base,
            },
        ];
        let clusters = cluster_commits(commits);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn test_cluster_empty() {
        let clusters = cluster_commits(Vec::new());
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_infer_intent_single() {
        let commits = vec![CommitInfo {
            sha: "aaa".into(),
            message: "Add user auth".into(),
            author: "Alice".into(),
            timestamp: 1000,
        }];
        let (intent, _summary) = infer_intent_description(&commits);
        assert_eq!(intent, "Add user auth");
    }

    #[test]
    fn test_infer_intent_multiple() {
        let commits = vec![
            CommitInfo {
                sha: "aaa".into(),
                message: "Add user auth".into(),
                author: "Alice".into(),
                timestamp: 1000,
            },
            CommitInfo {
                sha: "bbb".into(),
                message: "Fix auth tests".into(),
                author: "Alice".into(),
                timestamp: 2000,
            },
        ];
        let (intent, summary) = infer_intent_description(&commits);
        assert_eq!(intent, "Add user auth");
        assert!(summary.contains("2 commits"));
    }

    #[test]
    fn test_truncate_for_display() {
        assert_eq!(truncate_for_display("short", 60), "short");
        let long = "a".repeat(100);
        let truncated = truncate_for_display(&long, 20);
        assert!(truncated.len() <= 20);
        assert!(truncated.ends_with("..."));
    }
}

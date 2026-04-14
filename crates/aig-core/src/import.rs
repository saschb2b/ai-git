use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::db::Database;
use crate::git_interop::{self, CommitInfo};
use crate::intent;

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

    // 4. Initialize .aig/ database
    let db = Database::new()?;
    db.init_schema()?;

    // 5. For each cluster, create an intent and link commits as checkpoints
    let total_clusters = clusters.len();
    let mut total_commits: usize = 0;

    for (i, cluster) in clusters.iter().enumerate() {
        println!(
            "Importing intent {}/{}: \"{}\"",
            i + 1,
            total_clusters,
            truncate_for_display(&cluster.inferred_intent, 60)
        );

        let intent_id = intent::create_intent(&db, &cluster.inferred_intent)?;

        // Optionally store the summary on the intent
        if !cluster.summary.is_empty() {
            db.conn.execute(
                "UPDATE intents SET summary = ?1 WHERE id = ?2",
                rusqlite::params![cluster.summary, intent_id],
            )?;
        }

        // Insert a checkpoint for each commit in the cluster
        for commit in &cluster.commits {
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

        total_commits += cluster.commits.len();
    }

    // 6. Print summary
    println!(
        "Import complete: {} intents created from {} commits",
        total_clusters, total_commits
    );

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

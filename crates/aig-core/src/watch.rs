use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::checkpoint::CheckpointManager;
use crate::db::Database;
use crate::git_interop;
use crate::intent;
use crate::session::SessionManager;

/// Directories to ignore when watching for changes.
const IGNORED_DIRS: &[&str] = &[".git", ".aig", "target", "node_modules"];

/// Quiet period in seconds: auto-checkpoint after no changes for this long.
const QUIET_PERIOD_SECS: u64 = 30;

/// Check if a path should be ignored based on its components.
fn should_ignore(path: &Path) -> bool {
    for component in path.components() {
        let s = component.as_os_str().to_string_lossy();
        if IGNORED_DIRS.contains(&s.as_ref()) {
            return true;
        }
    }
    false
}

/// Check if an event kind is one we care about (create, modify content, remove).
fn is_relevant_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_)
            | EventKind::Modify(notify::event::ModifyKind::Data(_))
            | EventKind::Remove(_)
    )
}

/// Watch the working directory for file changes.
/// Collects changed files and reports them periodically.
/// When `auto_checkpoint` is true, automatically creates checkpoints
/// after a quiet period (no changes for 30 seconds).
pub fn watch_directory(repo_path: &str, auto_checkpoint: bool) -> Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(Path::new(repo_path), RecursiveMode::Recursive)?;

    println!("Watching for changes... (press Ctrl+C to stop)");
    if auto_checkpoint {
        println!(
            "  Auto-checkpoint enabled (quiet period: {}s)",
            QUIET_PERIOD_SECS
        );
    }

    let mut pending_changes: HashSet<String> = HashSet::new();
    let mut last_change_time: Option<Instant> = None;

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(event)) => {
                if !is_relevant_event(&event.kind) {
                    continue;
                }

                for path in &event.paths {
                    if should_ignore(path) {
                        continue;
                    }

                    let display_path = path
                        .strip_prefix(std::env::current_dir().unwrap_or_default())
                        .unwrap_or(path)
                        .to_string_lossy()
                        .replace('\\', "/");

                    if display_path.is_empty() {
                        continue;
                    }

                    println!("  modified: {}", display_path);
                    pending_changes.insert(display_path);
                    last_change_time = Some(Instant::now());
                }
            }
            Ok(Err(e)) => {
                eprintln!("  watch error: {e}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check if we've had a quiet period with pending changes
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }

        // Check quiet period
        if let Some(last_change) = last_change_time {
            if last_change.elapsed() >= Duration::from_secs(QUIET_PERIOD_SECS)
                && !pending_changes.is_empty()
            {
                let file_count = pending_changes.len();
                let file_list: Vec<&str> =
                    pending_changes.iter().take(5).map(|s| s.as_str()).collect();
                let file_summary = file_list.join(", ");
                let suffix = if file_count > 5 {
                    format!(", ... and {} more", file_count - 5)
                } else {
                    String::new()
                };

                println!();

                if auto_checkpoint {
                    match try_auto_checkpoint(repo_path, &pending_changes) {
                        Ok(true) => {
                            println!(
                                "Auto-checkpoint: {} files changed ({}{})",
                                file_count, file_summary, suffix
                            );
                        }
                        Ok(false) => {
                            println!(
                                "Changes detected: {} files modified. Run `aig checkpoint` to save.",
                                file_count
                            );
                            println!("  (no active session for auto-checkpoint)");
                        }
                        Err(e) => {
                            eprintln!("Auto-checkpoint failed: {e:#}");
                            println!(
                                "Changes detected: {} files modified. Run `aig checkpoint` to save.",
                                file_count
                            );
                        }
                    }
                } else {
                    println!(
                        "Changes detected: {} files modified. Run `aig checkpoint` to save.",
                        file_count
                    );
                }

                pending_changes.clear();
                last_change_time = None;
            }
        }
    }

    Ok(())
}

/// Attempt to create an auto-checkpoint.
/// Returns Ok(true) if a checkpoint was created, Ok(false) if no active session.
fn try_auto_checkpoint(repo_path: &str, changed_files: &HashSet<String>) -> Result<bool> {
    if !Path::new(".aig").exists() {
        return Ok(false);
    }

    let db = Database::new()?;
    let session = match SessionManager::get_active_session(&db)? {
        Some(s) => s,
        None => return Ok(false),
    };

    let intent_obj = intent::get_intent(&db, &session.intent_id)?;

    let file_count = changed_files.len();
    let file_list: Vec<&str> = changed_files.iter().take(5).map(|s| s.as_str()).collect();
    let file_summary = file_list.join(", ");
    let suffix = if file_count > 5 {
        ", ...".to_string()
    } else {
        String::new()
    };

    let message = format!(
        "Auto-checkpoint: {} files changed ({}{})",
        file_count, file_summary, suffix
    );

    let repo = git_interop::open_repo(repo_path)?;
    let git_sha = git_interop::create_commit(
        &repo,
        &format!("{}\n\naig intent: {}", message, intent_obj.description),
    )?;

    CheckpointManager::create_checkpoint(&db, &session.id, &message, &git_sha, &repo)?;

    Ok(true)
}

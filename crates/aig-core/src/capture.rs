use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::db::Database;

/// A parsed conversation entry from Claude Code
#[derive(Debug)]
pub struct ConversationEntry {
    pub role: String,    // "user" or "assistant"
    pub content: String, // the message text
}

/// Capture the current Claude Code conversation and import it into the given session.
/// Returns the number of entries imported, or 0 if nothing was found.
pub fn capture_conversation(db: &Database, session_id: &str) -> Result<usize> {
    let jsonl_path = match find_conversation_file()? {
        Some(p) => p,
        None => return Ok(0),
    };

    let entries = parse_conversation(&jsonl_path)?;
    if entries.is_empty() {
        return Ok(0);
    }

    let mut count = 0usize;
    for entry in &entries {
        let prefix = if entry.role == "user" {
            "[User]"
        } else {
            "[AI]"
        };
        let message = format!("{} {}", prefix, entry.content);

        let now = chrono::Utc::now();
        let id_input = format!(
            "capture-{}-{}-{}",
            session_id,
            count,
            now.timestamp_nanos_opt().unwrap_or(0)
        );
        let id = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(id_input.as_bytes());
            hex::encode(&hasher.finalize()[..16])
        };

        db.conn.execute(
            "INSERT INTO conversations (id, session_id, message, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, session_id, message, now.to_rfc3339()],
        )?;
        count += 1;
    }

    Ok(count)
}

/// Find the most recently modified .jsonl conversation file for the current project.
fn find_conversation_file() -> Result<Option<PathBuf>> {
    let cwd = std::env::current_dir()?;
    let cwd_str = cwd.to_string_lossy().to_string();

    // Convert the current working directory to Claude Code's project hash format.
    // E.g. "C:\Users\sasch\Documents\GitHub\ai-git" -> "C--Users-sasch-Documents-GitHub-ai-git"
    let project_hash = path_to_claude_hash(&cwd_str);

    // Look in ~/.claude/projects/<project_hash>/
    let home = match home_dir() {
        Some(h) => h,
        None => return Ok(None),
    };
    let projects_dir = home.join(".claude").join("projects");
    if !projects_dir.exists() {
        return Ok(None);
    }

    // Try exact match first
    let project_dir = projects_dir.join(&project_hash);
    if project_dir.is_dir() {
        return find_most_recent_jsonl(&project_dir);
    }

    // Fallback: scan directories for one containing the repo name
    let repo_name = cwd
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if repo_name.is_empty() {
        return Ok(None);
    }

    let entries = match std::fs::read_dir(&projects_dir) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };

    let mut best_dir: Option<PathBuf> = None;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(&format!("-{}", repo_name)) || name.contains(&project_hash) {
            let path = entry.path();
            if path.is_dir() {
                best_dir = Some(path);
                break;
            }
        }
    }

    match best_dir {
        Some(dir) => find_most_recent_jsonl(&dir),
        None => Ok(None),
    }
}

/// Convert an absolute path to Claude Code's project directory hash format.
fn path_to_claude_hash(path: &str) -> String {
    let mut result = path.to_string();
    // Replace drive letter separator: "C:\" or "C:/" -> "C--"
    result = result.replacen(":\\", "--", 1);
    result = result.replacen(":/", "--", 1);
    // Replace remaining path separators with "-"
    result = result.replace('\\', "-");
    result = result.replace('/', "-");
    result
}

/// Find the most recently modified .jsonl file in a directory.
fn find_most_recent_jsonl(dir: &Path) -> Result<Option<PathBuf>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(None),
    };

    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    if best.as_ref().is_none_or(|(_, t)| modified > *t) {
                        best = Some((path, modified));
                    }
                }
            }
        }
    }

    Ok(best.map(|(p, _)| p))
}

/// Parse a Claude Code .jsonl conversation file into meaningful entries.
fn parse_conversation(path: &Path) -> Result<Vec<ConversationEntry>> {
    let content = std::fs::read_to_string(path)?;
    let mut entries = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let obj: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if msg_type != "user" && msg_type != "assistant" {
            continue;
        }

        let message = match obj.get("message") {
            Some(m) => m,
            None => continue,
        };

        let role = message.get("role").and_then(|v| v.as_str()).unwrap_or("");
        if role != "user" && role != "assistant" {
            continue;
        }

        let text = extract_text_content(message);
        if text.is_empty() {
            continue;
        }

        // Clean the text
        let cleaned = clean_message(&text);
        if cleaned.is_empty() {
            continue;
        }

        // Filter out system-generated messages
        if is_system_message(&cleaned) {
            continue;
        }

        // For user messages, require minimum length
        if role == "user" && cleaned.len() < 10 {
            continue;
        }

        // Truncate to ~200 chars
        let truncated = truncate(&cleaned, 200);

        entries.push(ConversationEntry {
            role: role.to_string(),
            content: truncated,
        });
    }

    Ok(entries)
}

/// Extract text content from a message object.
/// Content can be a string or an array of content blocks.
fn extract_text_content(message: &Value) -> String {
    let content = match message.get("content") {
        Some(c) => c,
        None => return String::new(),
    };

    if let Some(s) = content.as_str() {
        return s.to_string();
    }

    if let Some(arr) = content.as_array() {
        let texts: Vec<String> = arr
            .iter()
            .filter_map(|block| {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    block.get("text").and_then(|t| t.as_str()).map(String::from)
                } else {
                    None
                }
            })
            .collect();
        return texts.join("\n");
    }

    String::new()
}

/// Remove XML tags and system content from a message.
fn clean_message(text: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut depth: i32 = 0;
    let skip_tags = [
        "system-reminder",
        "context",
        "task-notification",
        "local-command-stdout",
        "local-command-stderr",
        "local-command-result",
        "antml_",
    ];

    // Simple approach: strip lines that look like XML system content
    for line in text.lines() {
        let trimmed = line.trim();

        // Skip lines that are XML tags we want to filter
        if trimmed.starts_with('<') {
            let is_skip = skip_tags.iter().any(|tag| {
                trimmed.starts_with(&format!("<{}", tag))
                    || trimmed.starts_with(&format!("</{}", tag))
            });
            if is_skip {
                in_tag = true;
                // Check if self-closing or opening tag
                if let Some(tag) = extract_tag_name(trimmed) {
                    if !trimmed.ends_with("/>") && !trimmed.starts_with("</") {
                        tag_name = tag;
                        depth += 1;
                    } else if trimmed.starts_with("</") {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            in_tag = false;
                            tag_name.clear();
                        }
                    }
                }
                continue;
            }
        }

        // If we're inside a skipped tag block, keep skipping
        if in_tag && depth > 0 {
            if trimmed.starts_with(&format!("</{}", tag_name)) {
                depth -= 1;
                if depth == 0 {
                    in_tag = false;
                    tag_name.clear();
                }
            }
            continue;
        }

        if !result.is_empty() && !trimmed.is_empty() {
            result.push(' ');
        }
        result.push_str(trimmed);
    }

    result.trim().to_string()
}

/// Extract the tag name from an XML-like tag string.
fn extract_tag_name(tag_str: &str) -> Option<String> {
    let s = tag_str.trim_start_matches("</").trim_start_matches('<');
    let end = s
        .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
        .unwrap_or(s.len());
    let name = &s[..end];
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Check if a message is system-generated rather than user-authored.
fn is_system_message(text: &str) -> bool {
    let trimmed = text.trim();
    // Messages that start with XML tags are likely system-generated
    if trimmed.starts_with('<') {
        return true;
    }
    false
}

/// Truncate a string to max_len characters, adding "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let boundary = s
            .char_indices()
            .take_while(|(i, _)| *i < max_len - 3)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(max_len - 3);
        format!("{}...", &s[..boundary])
    }
}

/// Get the user's home directory.
fn home_dir() -> Option<PathBuf> {
    // Try HOME first (Unix and Git Bash on Windows), then USERPROFILE (Windows)
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_claude_hash() {
        assert_eq!(
            path_to_claude_hash(r"C:\Users\sasch\Documents\GitHub\ai-git"),
            "C--Users-sasch-Documents-GitHub-ai-git"
        );
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world this is long", 15), "hello world ...");
    }

    #[test]
    fn test_is_system_message() {
        assert!(is_system_message("<system-reminder>foo</system-reminder>"));
        assert!(!is_system_message("Hello, can you help me?"));
    }

    #[test]
    fn test_extract_text_content_string() {
        let msg: Value = serde_json::json!({
            "role": "user",
            "content": "Hello world"
        });
        assert_eq!(extract_text_content(&msg), "Hello world");
    }

    #[test]
    fn test_extract_text_content_array() {
        let msg: Value = serde_json::json!({
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Hello"},
                {"type": "tool_use", "id": "123"},
                {"type": "text", "text": "World"}
            ]
        });
        assert_eq!(extract_text_content(&msg), "Hello\nWorld");
    }

    #[test]
    fn test_clean_message() {
        let input =
            "<system-reminder>\nsome system stuff\n</system-reminder>\nActual user message here";
        let cleaned = clean_message(input);
        assert_eq!(cleaned, "Actual user message here");
    }
}

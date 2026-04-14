use anyhow::Result;
use std::fmt::Write;

/// Simple line-based diff between two strings
/// Returns a unified-diff-style output
pub fn line_diff(old: &str, new: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // Simple LCS-based diff
    let lcs = lcs_table(&old_lines, &new_lines);
    let mut output = String::new();
    let mut i = old_lines.len();
    let mut j = new_lines.len();

    let mut changes: Vec<DiffLine> = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            changes.push(DiffLine::Context(old_lines[i - 1]));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
            changes.push(DiffLine::Added(new_lines[j - 1]));
            j -= 1;
        } else if i > 0 {
            changes.push(DiffLine::Removed(old_lines[i - 1]));
            i -= 1;
        }
    }

    changes.reverse();

    for change in &changes {
        match change {
            DiffLine::Context(line) => {
                let _ = writeln!(output, "  {line}");
            }
            DiffLine::Added(line) => {
                let _ = writeln!(output, "+ {line}");
            }
            DiffLine::Removed(line) => {
                let _ = writeln!(output, "- {line}");
            }
        }
    }

    output
}

enum DiffLine<'a> {
    Context(&'a str),
    Added(&'a str),
    Removed(&'a str),
}

fn lcs_table(a: &[&str], b: &[&str]) -> Vec<Vec<usize>> {
    let m = a.len();
    let n = b.len();
    let mut table = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                table[i][j] = table[i - 1][j - 1] + 1;
            } else {
                table[i][j] = std::cmp::max(table[i - 1][j], table[i][j - 1]);
            }
        }
    }

    table
}

/// Get the working directory diff using git
/// Returns a list of (file_path, old_content, new_content) tuples for changed files
pub fn get_working_changes(repo: &git2::Repository) -> Result<Vec<ChangedFile>> {
    let mut changed = Vec::new();

    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

    let diff = repo.diff_tree_to_workdir_with_index(
        head_tree.as_ref(),
        Some(git2::DiffOptions::new().include_untracked(true)),
    )?;

    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .unwrap_or_else(|| std::path::Path::new(""))
            .to_string_lossy()
            .to_string();

        let old_content = if let Some(old_id) = Some(delta.old_file().id()) {
            if old_id.is_zero() {
                String::new()
            } else {
                repo.find_blob(old_id)
                    .map(|b| String::from_utf8_lossy(b.content()).to_string())
                    .unwrap_or_default()
            }
        } else {
            String::new()
        };

        let workdir = repo.workdir().unwrap_or_else(|| std::path::Path::new("."));
        let new_content = std::fs::read_to_string(workdir.join(&path)).unwrap_or_default();

        if old_content != new_content {
            changed.push(ChangedFile {
                path,
                old_content,
                new_content,
            });
        }
    }

    Ok(changed)
}

#[derive(Debug)]
pub struct ChangedFile {
    pub path: String,
    pub old_content: String,
    pub new_content: String,
}

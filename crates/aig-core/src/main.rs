use clap::{Parser, Subcommand};
use std::path::Path;

use aig_core::checkpoint::CheckpointManager;
use aig_core::db::Database;
use aig_core::diff;
use aig_core::git_interop;
use aig_core::intent;
use aig_core::session::SessionManager;
use aig_core::storage::BlobStore;

#[derive(Parser)]
#[command(
    name = "aig",
    about = "AI-native version control for intent-driven development"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new .aig directory in the current repo
    Init,
    /// Manage development sessions
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    /// Create a checkpoint (auto-generates message from semantic diff if omitted)
    Checkpoint {
        /// Checkpoint message (optional — auto-generated from changes if omitted)
        message: Option<String>,
    },
    /// Show current aig status
    Status,
    /// Show intent-level history
    Log,
    /// Show changes since last checkpoint
    Diff {
        /// Use semantic (tree-sitter) diff instead of line diff
        #[arg(long)]
        semantic: bool,
    },
    /// Explain why a line/region was changed
    Why {
        /// Location in the form "src/main.rs:42"
        location: String,
    },
    /// Import existing git history into aig
    Import,
    /// Manage conversation records
    Conversation {
        #[command(subcommand)]
        action: ConversationAction,
    },
    /// Watch for file changes and auto-checkpoint
    Watch {
        /// Automatically create checkpoints after quiet periods
        #[arg(long)]
        auto_checkpoint: bool,
    },
    /// Capture AI conversation into the active session
    Capture {
        /// Source to capture from: auto (default), claude-code, or a file path
        #[arg(long, default_value = "auto")]
        source: String,
        /// Import conversation from a file (JSONL with role/content per line)
        #[arg(long)]
        file: Option<String>,
    },
    /// Push aig metadata to remote via git notes
    Push {
        /// Remote name (default: origin)
        #[arg(default_value = "origin")]
        remote: String,
    },
    /// Pull aig metadata from remote via git notes
    Pull {
        /// Remote name (default: origin)
        #[arg(default_value = "origin")]
        remote: String,
    },
    /// Review an intent — show summary, semantic changes, and conversation
    Review {
        /// Intent ID (first 8 chars). Omit to review the most recent intent.
        intent_id: Option<String>,
    },
    /// Repair aig metadata after rebase (re-attaches orphaned notes)
    Repair,
}

#[derive(Subcommand)]
enum SessionAction {
    /// Start a new session with an intent
    Start {
        /// Description of the development intent
        intent: String,
    },
    /// End the current session
    End,
}

#[derive(Subcommand)]
enum ConversationAction {
    /// Add a conversation message to the current session
    Add {
        /// The message content
        message: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => cmd_init(),
        Commands::Session { action } => match action {
            SessionAction::Start { intent } => cmd_session_start(&intent),
            SessionAction::End => cmd_session_end(),
        },
        Commands::Checkpoint { message } => cmd_checkpoint(message.as_deref()),
        Commands::Status => cmd_status(),
        Commands::Log => cmd_log(),
        Commands::Diff { semantic } => cmd_diff(semantic),
        Commands::Why { location } => cmd_why(&location),
        Commands::Import => cmd_import(),
        Commands::Conversation { action } => match action {
            ConversationAction::Add { message } => cmd_conversation_add(&message),
        },
        Commands::Watch { auto_checkpoint } => cmd_watch(auto_checkpoint),
        Commands::Capture { source, file } => cmd_capture(&source, file.as_deref()),
        Commands::Push { remote } => cmd_push(&remote),
        Commands::Pull { remote } => cmd_pull(&remote),
        Commands::Review { intent_id } => cmd_review(intent_id.as_deref()),
        Commands::Repair => cmd_repair(),
    };

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        std::process::exit(1);
    }
}

fn ensure_aig_initialized() -> anyhow::Result<()> {
    if !Path::new(".aig").exists() {
        anyhow::bail!("not an aig repository (run `aig init` first)");
    }
    Ok(())
}

fn cmd_init() -> anyhow::Result<()> {
    if Path::new(".aig").exists() {
        println!("aig already initialized in this directory");
        return Ok(());
    }

    // Check we're in a git repo
    git_interop::open_repo(".")?;

    // Create .aig directory structure
    std::fs::create_dir_all(".aig/objects")?;

    // Initialize database with schema
    let db = Database::new()?;
    db.init_schema()?;

    // Initialize blob store
    let _store = BlobStore::new(Path::new(".aig"))?;

    println!("Initialized aig in .aig/");
    println!("  database: .aig/aig.db");
    println!("  objects:  .aig/objects/");
    println!("\nStart a session with: aig session start \"your intent\"");
    Ok(())
}

fn cmd_session_start(intent_desc: &str) -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    // Check for existing active session
    if let Some(session) = SessionManager::get_active_session(&db)? {
        let existing_intent = intent::get_intent(&db, &session.intent_id)?;
        anyhow::bail!(
            "session already active: \"{}\" (started {})\nEnd it first with: aig session end",
            existing_intent.description,
            session.started_at
        );
    }

    // Create the intent
    let intent_id = intent::create_intent(&db, intent_desc)?;

    // Start a session linked to that intent
    let session_id = SessionManager::start_session(&db, &intent_id)?;

    println!("Session started");
    println!("  intent:  {intent_desc}");
    println!("  session: {}", &session_id[..12]);
    println!("\nMake your changes, then run: aig checkpoint \"what you accomplished\"");
    Ok(())
}

fn cmd_session_end() -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    let session = SessionManager::get_active_session(&db)?
        .ok_or_else(|| anyhow::anyhow!("no active session"))?;

    let intent_obj = intent::get_intent(&db, &session.intent_id)?;

    // Close the intent
    let now = chrono::Utc::now().to_rfc3339();
    db.conn.execute(
        "UPDATE intents SET closed_at = ?1 WHERE id = ?2",
        rusqlite::params![now, session.intent_id],
    )?;

    // Auto-capture AI conversation before ending the session
    match aig_core::capture::capture_conversation(&db, &session.id, aig_core::capture::Source::Auto)
    {
        Ok((0, _)) => {} // No conversation found, silently skip
        Ok((count, source_name)) => {
            println!("Auto-captured {count} conversation entries from {source_name}");
        }
        Err(_) => {} // Silently skip capture errors
    }

    // End the session
    SessionManager::end_session(&db, &session.id)?;

    // Count checkpoints in this session
    let checkpoint_count: i64 = db.conn.query_row(
        "SELECT COUNT(*) FROM checkpoints WHERE session_id = ?1",
        rusqlite::params![session.id],
        |row| row.get(0),
    )?;

    println!("Session ended");
    println!("  intent:      {}", intent_obj.description);
    println!("  checkpoints: {checkpoint_count}");
    println!("  duration:    {} -> {now}", session.started_at);
    Ok(())
}

fn auto_generate_checkpoint_message(repo: &git2::Repository) -> String {
    // Generate a message from semantic diff of working changes
    let changes = diff::get_working_changes(repo).unwrap_or_default();
    let mut parts: Vec<String> = Vec::new();

    for change in &changes {
        let lang = aig_treesitter::detect_language(&change.path);
        if lang == aig_treesitter::Language::Unknown {
            continue;
        }
        if let Ok(sem_changes) =
            aig_treesitter::semantic_diff(&change.old_content, &change.new_content, lang)
        {
            for sc in &sem_changes {
                if !sc.symbol_name.is_empty() {
                    parts.push(format!("{} {}", sc.change_type, sc.symbol_name));
                }
            }
        }
    }

    if parts.is_empty() {
        // Fall back to file names
        let files: Vec<&str> = changes.iter().map(|c| c.path.as_str()).collect();
        if files.is_empty() {
            "Checkpoint".to_string()
        } else if files.len() <= 3 {
            format!("Update {}", files.join(", "))
        } else {
            format!("Update {} files", files.len())
        }
    } else if parts.len() <= 4 {
        parts.join(", ")
    } else {
        let first = parts[..3].join(", ");
        format!("{}, +{} more changes", first, parts.len() - 3)
    }
}

fn cmd_checkpoint(message: Option<&str>) -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    let session = SessionManager::get_active_session(&db)?.ok_or_else(|| {
        anyhow::anyhow!("no active session — start one with: aig session start \"intent\"")
    })?;

    let intent_obj = intent::get_intent(&db, &session.intent_id)?;

    // Auto-generate message from semantic diff if not provided
    let repo = git_interop::open_repo(".")?;
    let message = match message {
        Some(m) => m.to_string(),
        None => {
            let generated = auto_generate_checkpoint_message(&repo);
            println!("  auto-message: {generated}");
            generated
        }
    };

    // Create a git commit
    let git_sha = git_interop::create_commit(
        &repo,
        &format!("{}\n\naig intent: {}", message, intent_obj.description),
    )?;

    // Record the checkpoint in aig (also stores semantic changes via tree-sitter)
    let checkpoint_id =
        CheckpointManager::create_checkpoint(&db, &session.id, &message, &git_sha, &repo)?;

    let short_sha = &git_sha[..8];
    let short_id = &checkpoint_id[..12];

    // Show semantic changes that were recorded
    let mut sc_stmt = db.conn.prepare(
        "SELECT file_path, change_type, symbol_name FROM semantic_changes WHERE checkpoint_id = ?1",
    )?;
    let sem_changes: Vec<_> = sc_stmt
        .query_map(rusqlite::params![checkpoint_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    if !sem_changes.is_empty() {
        println!("  semantic:");
        for (file, change_type, symbol) in &sem_changes {
            println!(
                "    {} {} {symbol} ({file})",
                change_type_icon(change_type),
                change_type
            );
        }
    }

    println!("Checkpoint created");
    println!("  message:    {message}");
    println!("  intent:     {}", intent_obj.description);
    println!("  git commit: {short_sha}");
    println!("  checkpoint: {short_id}");
    Ok(())
}

fn cmd_status() -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    match SessionManager::get_active_session(&db)? {
        Some(session) => {
            let intent_obj = intent::get_intent(&db, &session.intent_id)?;

            let checkpoint_count: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM checkpoints WHERE session_id = ?1",
                rusqlite::params![session.id],
                |row| row.get(0),
            )?;

            let conversation_count: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM conversations WHERE session_id = ?1",
                rusqlite::params![session.id],
                |row| row.get(0),
            )?;

            println!("Active session");
            println!("  intent:        {}", intent_obj.description);
            println!("  session:       {}", &session.id[..12]);
            println!("  started:       {}", session.started_at);
            println!("  checkpoints:   {checkpoint_count}");
            println!("  conversations: {conversation_count}");

            // Show git working tree status
            let repo = git_interop::open_repo(".")?;
            let changes = diff::get_working_changes(&repo)?;
            if changes.is_empty() {
                println!("\n  working tree clean");
            } else {
                println!("\n  modified files:");
                for change in &changes {
                    println!("    {}", change.path);
                }
            }
        }
        None => {
            // Show summary even without active session
            let intent_count: i64 =
                db.conn
                    .query_row("SELECT COUNT(*) FROM intents", [], |row| row.get(0))?;
            let checkpoint_count: i64 =
                db.conn
                    .query_row("SELECT COUNT(*) FROM checkpoints", [], |row| row.get(0))?;

            println!("No active session");
            println!("  total intents:     {intent_count}");
            println!("  total checkpoints: {checkpoint_count}");
            println!("\nStart a session with: aig session start \"your intent\"");
        }
    }

    Ok(())
}

fn cmd_log() -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    let intents = intent::list_intents(&db)?;

    if intents.is_empty() {
        println!("No intents recorded yet.");
        println!("Start with: aig session start \"your intent\"");
        return Ok(());
    }

    for intent_obj in &intents {
        let short_id = &intent_obj.id[..8];

        // Count checkpoints for this intent
        let checkpoint_count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM checkpoints WHERE intent_id = ?1",
            rusqlite::params![intent_obj.id],
            |row| row.get(0),
        )?;

        // Get files touched via checkpoints -> git commits
        let status = if intent_obj.closed_at.is_some() {
            "done"
        } else {
            "active"
        };

        println!("[{short_id}] {} ({status})", intent_obj.description);
        println!(
            "         {checkpoint_count} checkpoint(s) | {}",
            intent_obj.created_at
        );

        // Show checkpoint messages
        let mut stmt = db.conn.prepare(
            "SELECT message, git_commit_sha, created_at FROM checkpoints WHERE intent_id = ?1 ORDER BY created_at",
        )?;
        let checkpoints = stmt.query_map(rusqlite::params![intent_obj.id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        for cp in checkpoints {
            let (msg, sha, _ts) = cp?;
            let short_sha = &sha[..8];
            println!("           ({short_sha}) {msg}");

            // Show semantic changes for this checkpoint
            let cp_id_result: Result<String, _> = db.conn.query_row(
                "SELECT id FROM checkpoints WHERE git_commit_sha = ?1",
                rusqlite::params![sha],
                |row| row.get(0),
            );
            if let Ok(cp_id) = cp_id_result {
                let mut sc_stmt = db.conn.prepare(
                    "SELECT change_type, symbol_name, file_path FROM semantic_changes WHERE checkpoint_id = ?1",
                )?;
                let scs: Vec<_> = sc_stmt
                    .query_map(rusqlite::params![cp_id], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                for (ct, sym, fp) in &scs {
                    println!(
                        "                     {} {} `{sym}` ({fp})",
                        change_type_icon(ct),
                        ct
                    );
                }
            }
        }
        println!();
    }

    Ok(())
}

fn cmd_diff(semantic: bool) -> anyhow::Result<()> {
    ensure_aig_initialized()?;

    let repo = git_interop::open_repo(".")?;
    let changes = diff::get_working_changes(&repo)?;

    if changes.is_empty() {
        println!("No changes since last checkpoint.");
        return Ok(());
    }

    if semantic {
        // Semantic diff using tree-sitter
        for change in &changes {
            let lang = aig_treesitter::detect_language(&change.path);
            if lang == aig_treesitter::Language::Unknown {
                println!(
                    "--- {} (no semantic diff for this file type, showing line diff)",
                    change.path
                );
                print!(
                    "{}",
                    diff::line_diff(&change.old_content, &change.new_content)
                );
                println!();
                continue;
            }

            match aig_treesitter::semantic_diff(&change.old_content, &change.new_content, lang) {
                Ok(semantic_changes) if !semantic_changes.is_empty() => {
                    println!("--- {} (semantic)", change.path);
                    for sc in &semantic_changes {
                        let symbol = if sc.symbol_name.is_empty() {
                            String::new()
                        } else {
                            format!(" `{}`", sc.symbol_name)
                        };
                        let details = if sc.details.is_empty() {
                            String::new()
                        } else {
                            format!(" — {}", sc.details)
                        };
                        println!(
                            "  {} {}{}{}",
                            change_type_icon(&sc.change_type),
                            sc.change_type,
                            symbol,
                            details
                        );
                    }
                    println!();
                }
                Ok(_) => {
                    println!("--- {} (no semantic changes detected)", change.path);
                    println!();
                }
                Err(_) => {
                    println!(
                        "--- {} (semantic diff failed, showing line diff)",
                        change.path
                    );
                    print!(
                        "{}",
                        diff::line_diff(&change.old_content, &change.new_content)
                    );
                    println!();
                }
            }
        }
    } else {
        // Line-based diff
        for change in &changes {
            println!("--- {}", change.path);
            print!(
                "{}",
                diff::line_diff(&change.old_content, &change.new_content)
            );
            println!();
        }
    }

    Ok(())
}

fn change_type_icon(change_type: &str) -> &str {
    match change_type {
        "added" => "+",
        "removed" => "-",
        "modified" => "~",
        _ => "?",
    }
}

fn cmd_why(location: &str) -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    // Parse location "file:line"
    let (file_path, line_num) = match location.rsplit_once(':') {
        Some((f, l)) => match l.parse::<usize>() {
            Ok(n) => (f, n),
            Err(_) => (location, 0),
        },
        None => (location, 0),
    };

    // Find checkpoints that touched this file
    // We search checkpoints -> git commits, then check if the commit changed this file
    let repo = git_interop::open_repo(".")?;

    let mut stmt = db.conn.prepare(
        "SELECT c.id, c.message, c.git_commit_sha, c.created_at, i.description, i.id
         FROM checkpoints c
         JOIN intents i ON c.intent_id = i.id
         ORDER BY c.created_at DESC",
    )?;

    let rows: Vec<_> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Find the most recent checkpoint whose git commit touched this file
    for (_cp_id, cp_msg, git_sha, cp_time, intent_desc, intent_id) in &rows {
        let oid = git2::Oid::from_str(git_sha)?;
        let commit = repo.find_commit(oid)?;

        // Get the diff for this commit
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
        let commit_tree = commit.tree()?;
        let diff_result = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)?;

        let mut touches_file = false;
        for delta in diff_result.deltas() {
            let path = delta.new_file().path().unwrap_or(Path::new(""));
            if path.to_string_lossy() == file_path {
                touches_file = true;
                break;
            }
        }

        if touches_file {
            let short_sha = &git_sha[..8];
            let short_intent = &intent_id[..8];

            if line_num > 0 {
                println!("{}:{}", file_path, line_num);
            } else {
                println!("{}", file_path);
            }
            println!();
            println!("  Intent:     [{short_intent}] {intent_desc}");
            println!("  Checkpoint: {cp_msg}");
            println!("  Commit:     {short_sha}");
            println!("  Time:       {cp_time}");

            // Show semantic changes for this checkpoint on this file
            let mut sc_stmt = db.conn.prepare(
                "SELECT change_type, symbol_name, details FROM semantic_changes
                 WHERE checkpoint_id = ?1 AND file_path = ?2",
            )?;
            let sem_changes: Vec<_> = sc_stmt
                .query_map(rusqlite::params![_cp_id, file_path], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            if !sem_changes.is_empty() {
                println!();
                println!("  Semantic changes:");
                for (change_type, symbol, details) in &sem_changes {
                    let icon = change_type_icon(change_type);
                    let detail_str = if details.is_empty() {
                        String::new()
                    } else {
                        format!(" — {details}")
                    };
                    println!("    {icon} {change_type} `{symbol}`{detail_str}");
                }
            }

            // Show conversation notes for this session
            let conv_count: i64 = db.conn.query_row(
                "SELECT COUNT(*) FROM conversations c
                 JOIN sessions s ON c.session_id = s.id
                 WHERE s.intent_id = ?1",
                rusqlite::params![intent_id],
                |row| row.get(0),
            )?;

            if conv_count > 0 {
                println!();
                println!("  Conversation notes:");
                let mut conv_stmt = db.conn.prepare(
                    "SELECT c.message, c.created_at FROM conversations c
                     JOIN sessions s ON c.session_id = s.id
                     WHERE s.intent_id = ?1
                     ORDER BY c.created_at",
                )?;
                let convs = conv_stmt.query_map(rusqlite::params![intent_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;

                for conv in convs {
                    let (msg, _ts) = conv?;
                    println!("    - {msg}");
                }
            }

            return Ok(());
        }
    }

    println!("No aig history found for {file_path}");
    println!("This file may predate aig tracking, or was not changed in any tracked session.");
    Ok(())
}

fn cmd_import() -> anyhow::Result<()> {
    // Initialize aig if needed
    if !Path::new(".aig").exists() {
        cmd_init()?;
    }

    aig_core::import::import_git_history(".")?;
    Ok(())
}

fn cmd_conversation_add(message: &str) -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    let session = SessionManager::get_active_session(&db)?.ok_or_else(|| {
        anyhow::anyhow!("no active session — start one with: aig session start \"intent\"")
    })?;

    // Generate an id for the conversation entry
    let now = chrono::Utc::now();
    let id_input = format!(
        "conv-{}-{}",
        message,
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
        rusqlite::params![id, session.id, message, now.to_rfc3339()],
    )?;

    let intent_obj = intent::get_intent(&db, &session.intent_id)?;
    println!("Conversation note added to session");
    println!("  intent: {}", intent_obj.description);
    println!("  note:   {message}");
    Ok(())
}

fn cmd_watch(auto_checkpoint: bool) -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    aig_core::watch::watch_directory(".", auto_checkpoint)
}

fn cmd_capture(source_arg: &str, file_arg: Option<&str>) -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    let session = SessionManager::get_active_session(&db)?.ok_or_else(|| {
        anyhow::anyhow!("no active session — start one with: aig session start \"intent\"")
    })?;

    let intent_obj = intent::get_intent(&db, &session.intent_id)?;

    let source = if let Some(path) = file_arg {
        aig_core::capture::Source::File(std::path::PathBuf::from(path))
    } else {
        match source_arg {
            "claude-code" => aig_core::capture::Source::ClaudeCode,
            "auto" => aig_core::capture::Source::Auto,
            other => {
                // Treat unknown source values as file paths for convenience
                aig_core::capture::Source::File(std::path::PathBuf::from(other))
            }
        }
    };

    match aig_core::capture::capture_conversation(&db, &session.id, source) {
        Ok((0, _)) => {
            println!("No AI conversation found. Try --file to import manually.");
        }
        Ok((count, source_name)) => {
            println!("Captured {count} conversation entries from {source_name}");
            println!("  intent:  {}", intent_obj.description);
            println!("  session: {}", &session.id[..12]);
        }
        Err(e) => {
            println!("Could not capture conversation: {e}");
        }
    }

    Ok(())
}

fn cmd_push(remote: &str) -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;
    let repo = git_interop::open_repo(".")?;

    println!("Writing aig metadata to git notes...");
    let count = aig_core::sync::push_notes(&db, &repo)?;

    if count == 0 {
        println!("No checkpoints to push.");
        return Ok(());
    }

    println!("Pushing refs/notes/aig to {remote}...");
    aig_core::sync::push_to_remote(".", remote)?;

    println!("Pushed {count} checkpoint(s) to {remote}");
    Ok(())
}

fn cmd_pull(remote: &str) -> anyhow::Result<()> {
    // Initialize aig if needed (pulling into a fresh clone)
    if !Path::new(".aig").exists() {
        cmd_init()?;
    }

    let db = Database::new()?;
    let repo = git_interop::open_repo(".")?;

    println!("Fetching refs/notes/aig from {remote}...");
    aig_core::sync::pull_from_remote(".", remote)?;

    println!("Importing aig metadata from git notes...");
    let count = aig_core::sync::pull_notes(&db, &repo)?;

    if count == 0 {
        println!("No aig metadata found in remote notes.");
    } else {
        println!("Imported {count} checkpoint(s) from {remote}");
    }

    Ok(())
}

fn cmd_review(intent_id: Option<&str>) -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;

    let intents = intent::list_intents(&db)?;
    if intents.is_empty() {
        println!("No intents recorded yet.");
        println!("Start with: aig session start \"your intent\"");
        return Ok(());
    }

    // Find the target intent
    let intent_obj = if let Some(prefix) = intent_id {
        intents
            .into_iter()
            .find(|i| i.id.starts_with(prefix))
            .ok_or_else(|| anyhow::anyhow!("no intent found matching prefix \"{prefix}\""))?
    } else {
        // Most recent intent (list_intents is DESC by created_at)
        intents.into_iter().next().unwrap()
    };

    // Header
    let status = if intent_obj.closed_at.is_some() {
        "done"
    } else {
        "active"
    };
    println!("Review: {}", intent_obj.description);
    println!("Status: {status}");

    // Duration
    let start_display = format_datetime(&intent_obj.created_at);
    if let Some(ref closed) = intent_obj.closed_at {
        let end_display = format_datetime(closed);
        let duration = compute_duration(&intent_obj.created_at, closed);
        println!("Duration: {start_display} — {end_display} ({duration})");
    } else {
        let now = chrono::Utc::now().to_rfc3339();
        let duration = compute_duration(&intent_obj.created_at, &now);
        println!("Duration: {start_display} — now ({duration})");
    }

    // Checkpoints
    let mut cp_stmt = db.conn.prepare(
        "SELECT id, message, git_commit_sha, created_at FROM checkpoints WHERE intent_id = ?1 ORDER BY created_at",
    )?;
    let checkpoints: Vec<(String, String, String, String)> = cp_stmt
        .query_map(rusqlite::params![intent_obj.id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    println!();
    println!("Checkpoints ({}):", checkpoints.len());
    for (i, (_cp_id, msg, sha, _ts)) in checkpoints.iter().enumerate() {
        let short_sha = if sha.len() >= 8 { &sha[..8] } else { sha };
        println!("  {}. ({short_sha}) {msg}", i + 1);
    }

    // Semantic changes: aggregate across all checkpoints, group by file
    // For each (file_path, symbol_name), keep the latest change_type
    use std::collections::BTreeMap;

    // BTreeMap<file_path, Vec<(symbol_name, change_type)>> — dedup by symbol, last wins
    let mut file_changes: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

    for (cp_id, _msg, _sha, _ts) in &checkpoints {
        let mut sc_stmt = db.conn.prepare(
            "SELECT file_path, change_type, symbol_name FROM semantic_changes WHERE checkpoint_id = ?1",
        )?;
        let scs: Vec<_> = sc_stmt
            .query_map(rusqlite::params![cp_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        for (file_path, change_type, symbol_name) in scs {
            let entry = file_changes.entry(file_path).or_default();
            // Deduplicate by symbol_name, keeping latest change_type
            if let Some(existing) = entry.iter_mut().find(|(s, _)| s == &symbol_name) {
                existing.1 = change_type;
            } else {
                entry.push((symbol_name, change_type));
            }
        }
    }

    if !file_changes.is_empty() {
        println!();
        println!("Semantic changes:");
        for (file_path, symbols) in &file_changes {
            println!("  {file_path}");
            for (symbol_name, change_type) in symbols {
                let icon = change_type_icon(change_type);
                println!("    {icon} {change_type} `{symbol_name}`");
            }
        }
    }

    // Files touched: collect unique file paths from git commits
    let mut files_touched = Vec::new();
    let repo = git_interop::open_repo(".")?;

    for (_cp_id, _msg, sha, _ts) in &checkpoints {
        if let Ok(oid) = git2::Oid::from_str(sha) {
            if let Ok(commit) = repo.find_commit(oid) {
                let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
                let commit_tree = match commit.tree() {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if let Ok(diff_result) =
                    repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)
                {
                    for delta in diff_result.deltas() {
                        let path = delta.new_file().path().unwrap_or(Path::new(""));
                        let path_str = path.to_string_lossy().to_string();
                        if !files_touched.contains(&path_str) {
                            files_touched.push(path_str);
                        }
                    }
                }
            }
        }
    }

    files_touched.sort();

    println!();
    println!("Files touched: {}", files_touched.len());
    for f in &files_touched {
        println!("  {f}");
    }

    // Conversation notes: query all sessions for this intent, then their conversations
    let mut conv_stmt = db.conn.prepare(
        "SELECT c.message FROM conversations c
         JOIN sessions s ON c.session_id = s.id
         WHERE s.intent_id = ?1
         ORDER BY c.created_at",
    )?;
    let conversations: Vec<String> = conv_stmt
        .query_map(rusqlite::params![intent_obj.id], |row| {
            row.get::<_, String>(0)
        })?
        .collect::<Result<Vec<_>, _>>()?;

    if !conversations.is_empty() {
        println!();
        println!("Conversation ({} notes):", conversations.len());
        for msg in &conversations {
            println!("  - {msg}");
        }
    }

    Ok(())
}

fn cmd_repair() -> anyhow::Result<()> {
    ensure_aig_initialized()?;
    let db = Database::new()?;
    let repo = git_interop::open_repo(".")?;

    println!("Scanning for orphaned notes...");
    let result = aig_core::repair::repair_notes(&db, &repo)?;

    println!("Repair complete:");
    println!("  ok:       {} notes still valid", result.ok);
    println!("  repaired: {} notes re-attached", result.repaired);
    if result.orphaned > 0 {
        println!("  orphaned: {} notes could not be matched", result.orphaned);
    }
    Ok(())
}

fn format_datetime(rfc3339: &str) -> String {
    use chrono::DateTime;
    match rfc3339.parse::<DateTime<chrono::Utc>>() {
        Ok(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        Err(_) => rfc3339.to_string(),
    }
}

fn compute_duration(start_rfc3339: &str, end_rfc3339: &str) -> String {
    use chrono::DateTime;
    let start = match start_rfc3339.parse::<DateTime<chrono::Utc>>() {
        Ok(dt) => dt,
        Err(_) => return "unknown".to_string(),
    };
    let end = match end_rfc3339.parse::<DateTime<chrono::Utc>>() {
        Ok(dt) => dt,
        Err(_) => return "unknown".to_string(),
    };
    let duration = end.signed_duration_since(start);
    let total_secs = duration.num_seconds();
    if total_secs < 0 {
        return "unknown".to_string();
    }
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let mins = (total_secs % 3600) / 60;

    if days > 0 {
        if days == 1 {
            "1 day".to_string()
        } else {
            format!("{days} days")
        }
    } else if hours > 0 {
        if hours == 1 && mins == 0 {
            "1 hour".to_string()
        } else if mins == 0 {
            format!("{hours} hours")
        } else {
            format!("{hours} h {mins} min")
        }
    } else if mins > 0 {
        format!("{mins} min")
    } else {
        format!("{total_secs} sec")
    }
}

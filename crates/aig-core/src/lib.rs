/// Export/import `.aig` metadata as portable tar.gz bundles.
pub mod bundle;
/// Auto-capture AI conversations (Claude Code, generic JSONL).
pub mod capture;
/// Create checkpoints (git commit + aig metadata + provenance).
pub mod checkpoint;
/// SQLite database schema and connection.
pub mod db;
/// Line-based and semantic diff orchestration.
pub mod diff;
/// Git operations via git2 (repo access, commit creation).
pub mod git_interop;
/// Import existing git history into aig + IPC client for LLM calls.
pub mod import;
/// Intent CRUD — the "why" behind changes.
pub mod intent;
/// Re-attach orphaned metadata after rebase/cherry-pick.
pub mod repair;
/// Start/end development sessions tied to intents.
pub mod session;
/// Content-addressable blob store (`.aig/objects/`).
pub mod storage;
/// Push/pull aig metadata via git notes.
pub mod sync;
/// File-system watcher for auto-checkpointing.
pub mod watch;

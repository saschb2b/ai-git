use std::path::Path;
use tempfile::tempdir;

use aig_core::checkpoint::CheckpointManager;
use aig_core::db::Database;
use aig_core::diff;
use aig_core::git_interop::{self, CommitInfo};
use aig_core::import::cluster_commits;
use aig_core::intent;
use aig_core::session::SessionManager;
use aig_core::storage::BlobStore;

/// Helper: create a Database backed by a file inside the given directory.
/// This avoids relying on the cwd-based `Database::new()`.
fn create_db(dir: &Path) -> Database {
    let aig_dir = dir.join(".aig");
    std::fs::create_dir_all(&aig_dir).unwrap();
    let db_path = aig_dir.join("aig.db");
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    Database { conn }
}

/// Helper: initialize a git repository with an initial empty commit.
fn init_test_repo(dir: &Path) -> git2::Repository {
    let repo = git2::Repository::init(dir).unwrap();
    {
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
    }
    repo
}

// ---------------------------------------------------------------------------
// Test 1: test_init_creates_aig_directory
// ---------------------------------------------------------------------------

#[test]
fn test_init_creates_aig_directory() {
    let dir = tempdir().unwrap();
    let _repo = git2::Repository::init(dir.path()).unwrap();

    // Simulate `aig init`: create Database and BlobStore
    let _db = create_db(dir.path());
    let _blob_store = BlobStore::new(&dir.path().join(".aig")).unwrap();

    // Verify .aig/aig.db exists
    assert!(dir.path().join(".aig").join("aig.db").exists());

    // Verify .aig/objects/ exists
    assert!(dir.path().join(".aig").join("objects").is_dir());
}

// ---------------------------------------------------------------------------
// Test 2: test_session_lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_session_lifecycle() {
    let dir = tempdir().unwrap();
    let _repo = init_test_repo(dir.path());
    let db = create_db(dir.path());
    db.init_schema().unwrap();

    // Create an intent
    let intent_id = intent::create_intent(&db, "add login feature").unwrap();
    assert!(!intent_id.is_empty());

    // Start a session
    let session_id = SessionManager::start_session(&db, &intent_id).unwrap();
    assert!(!session_id.is_empty());

    // Verify active session exists
    let active = SessionManager::get_active_session(&db).unwrap();
    assert!(active.is_some());
    let active = active.unwrap();
    assert_eq!(active.id, session_id);
    assert_eq!(active.intent_id, intent_id);
    assert!(active.ended_at.is_none());

    // End the session
    SessionManager::end_session(&db, &session_id).unwrap();

    // Verify no active session
    let active_after = SessionManager::get_active_session(&db).unwrap();
    assert!(active_after.is_none());
}

// ---------------------------------------------------------------------------
// Test 3: test_checkpoint_creates_git_commit
// ---------------------------------------------------------------------------

#[test]
fn test_checkpoint_creates_git_commit() {
    let dir = tempdir().unwrap();
    let repo = init_test_repo(dir.path());
    let db = create_db(dir.path());
    db.init_schema().unwrap();

    let intent_id = intent::create_intent(&db, "checkpoint test intent").unwrap();
    let session_id = SessionManager::start_session(&db, &intent_id).unwrap();

    // Write a file and create a git commit
    std::fs::write(dir.path().join("hello.txt"), "hello world").unwrap();
    let sha = git_interop::create_commit(&repo, "add hello.txt").unwrap();
    assert!(!sha.is_empty());

    // Create a checkpoint record
    let cp_id =
        CheckpointManager::create_checkpoint(&db, &session_id, "first checkpoint", &sha, &repo)
            .unwrap();
    assert!(!cp_id.is_empty());

    // Verify checkpoint is in the database
    let row: (String, String, String) = db
        .conn
        .query_row(
            "SELECT id, session_id, git_commit_sha FROM checkpoints WHERE id = ?1",
            rusqlite::params![cp_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(row.0, cp_id);
    assert_eq!(row.1, session_id);
    assert_eq!(row.2, sha);

    // Verify git log has the commit
    let log = git_interop::get_log(&repo, 10).unwrap();
    assert!(log.iter().any(|c| c.sha == sha));
    assert!(log.iter().any(|c| c.message.contains("add hello.txt")));
}

// ---------------------------------------------------------------------------
// Test 4: test_intent_history
// ---------------------------------------------------------------------------

#[test]
fn test_intent_history() {
    let dir = tempdir().unwrap();
    let db = create_db(dir.path());
    db.init_schema().unwrap();

    // Create multiple intents with slight delay to ensure ordering
    let id1 = intent::create_intent(&db, "first intent").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let id2 = intent::create_intent(&db, "second intent").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let id3 = intent::create_intent(&db, "third intent").unwrap();

    // list_intents returns reverse chronological order
    let intents = intent::list_intents(&db).unwrap();
    assert_eq!(intents.len(), 3);
    assert_eq!(intents[0].id, id3);
    assert_eq!(intents[1].id, id2);
    assert_eq!(intents[2].id, id1);

    // get_intent retrieves by ID correctly
    let fetched = intent::get_intent(&db, &id2).unwrap();
    assert_eq!(fetched.id, id2);
    assert_eq!(fetched.description, "second intent");
}

// ---------------------------------------------------------------------------
// Test 5: test_line_diff
// ---------------------------------------------------------------------------

#[test]
fn test_line_diff() {
    let old = "fn hello() {\n    println!(\"hi\");\n}";
    let new = "fn hello() {\n    println!(\"hello world\");\n}";

    let output = diff::line_diff(old, new);

    // Should contain a removed line and an added line for the changed println
    assert!(
        output.contains("- "),
        "diff output should contain a removed line marker: {output}"
    );
    assert!(
        output.contains("+ "),
        "diff output should contain an added line marker: {output}"
    );
    assert!(output.contains("hi"), "diff should reference old content");
    assert!(
        output.contains("hello world"),
        "diff should reference new content"
    );
}

// ---------------------------------------------------------------------------
// Test 6: test_semantic_diff_python
// ---------------------------------------------------------------------------

#[test]
fn test_semantic_diff_python() {
    let old = "def greet(name):\n    return f\"Hello {name}\"\n\ndef farewell():\n    pass";
    let new =
        "def greet(name, formal=False):\n    return f\"Hello {name}\"\n\ndef welcome():\n    pass";

    let changes =
        aig_treesitter::semantic_diff(old, new, aig_treesitter::Language::Python).unwrap();

    let greet_change = changes.iter().find(|c| c.symbol_name == "greet");
    assert!(
        greet_change.is_some(),
        "greet should appear in changes: {changes:?}"
    );
    assert_eq!(greet_change.unwrap().change_type, "modified");

    let farewell_change = changes.iter().find(|c| c.symbol_name == "farewell");
    assert!(
        farewell_change.is_some(),
        "farewell should appear in changes: {changes:?}"
    );
    assert_eq!(farewell_change.unwrap().change_type, "removed");

    let welcome_change = changes.iter().find(|c| c.symbol_name == "welcome");
    assert!(
        welcome_change.is_some(),
        "welcome should appear in changes: {changes:?}"
    );
    assert_eq!(welcome_change.unwrap().change_type, "added");
}

// ---------------------------------------------------------------------------
// Test 7: test_semantic_diff_typescript
// ---------------------------------------------------------------------------

#[test]
fn test_semantic_diff_typescript() {
    let old = "function add(a: number, b: number): number { return a + b; }";
    let new = "function add(a: number, b: number): number { return a + b; }\nfunction multiply(a: number, b: number): number { return a * b; }";

    let changes =
        aig_treesitter::semantic_diff(old, new, aig_treesitter::Language::TypeScript).unwrap();

    let multiply_change = changes.iter().find(|c| c.symbol_name == "multiply");
    assert!(
        multiply_change.is_some(),
        "multiply should appear in changes: {changes:?}"
    );
    assert_eq!(multiply_change.unwrap().change_type, "added");

    // add is unchanged -- should NOT appear in results
    let add_change = changes.iter().find(|c| c.symbol_name == "add");
    assert!(
        add_change.is_none(),
        "add should not appear in changes since it is unchanged: {changes:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 8: test_semantic_diff_rust
// ---------------------------------------------------------------------------

#[test]
fn test_semantic_diff_rust() {
    let old = "fn hello() { println!(\"hello\"); }\nstruct User { name: String }";
    let new =
        "fn hello() { println!(\"hi\"); }\nstruct User { name: String, age: u32 }\nfn goodbye() {}";

    let changes = aig_treesitter::semantic_diff(old, new, aig_treesitter::Language::Rust).unwrap();

    let hello_change = changes.iter().find(|c| c.symbol_name == "hello");
    assert!(
        hello_change.is_some(),
        "hello should appear in changes: {changes:?}"
    );
    assert_eq!(hello_change.unwrap().change_type, "modified");

    let user_change = changes.iter().find(|c| c.symbol_name == "User");
    assert!(
        user_change.is_some(),
        "User should appear in changes: {changes:?}"
    );
    assert_eq!(user_change.unwrap().change_type, "modified");

    let goodbye_change = changes.iter().find(|c| c.symbol_name == "goodbye");
    assert!(
        goodbye_change.is_some(),
        "goodbye should appear in changes: {changes:?}"
    );
    assert_eq!(goodbye_change.unwrap().change_type, "added");
}

// ---------------------------------------------------------------------------
// Test 9: test_import_clusters_commits
// ---------------------------------------------------------------------------

#[test]
fn test_import_clusters_commits() {
    let base = 1_000_000i64;

    // Commits are presented newest-first (as get_log returns them).
    // cluster_commits reverses to oldest-first before clustering.
    // After reverse the order will be: aaa, bbb, ccc, ddd.
    let commits = vec![
        // Bob's commit, 3.5 hours after Alice's last -- different author
        CommitInfo {
            sha: "ddd".into(),
            message: "Bob: docs update".into(),
            author: "Bob".into(),
            timestamp: base + 4 * 3600,
        },
        // Alice's third commit, 3+ hours after bbb -- new cluster (time gap)
        CommitInfo {
            sha: "ccc".into(),
            message: "refactor auth".into(),
            author: "Alice".into(),
            timestamp: base + 3 * 3600 + 1800,
        },
        // Alice's second commit, 30 minutes after the first -- same cluster
        CommitInfo {
            sha: "bbb".into(),
            message: "fix tests".into(),
            author: "Alice".into(),
            timestamp: base + 1800,
        },
        // Alice's first commit
        CommitInfo {
            sha: "aaa".into(),
            message: "add auth module".into(),
            author: "Alice".into(),
            timestamp: base,
        },
    ];

    let clusters = cluster_commits(commits);

    // After reversing to oldest-first:
    //   aaa (Alice, base)
    //   bbb (Alice, base+1800)  -- same cluster as aaa (gap < 2h)
    //   ccc (Alice, base+3.5h)  -- new cluster (gap from bbb > 2h)
    //   ddd (Bob, base+4h)      -- new cluster (different author)
    assert_eq!(clusters.len(), 3, "expected 3 clusters: {clusters:?}");

    // First cluster: aaa + bbb
    assert_eq!(clusters[0].commits.len(), 2);
    assert_eq!(clusters[0].commits[0].sha, "aaa");
    assert_eq!(clusters[0].commits[1].sha, "bbb");

    // Second cluster: ccc
    assert_eq!(clusters[1].commits.len(), 1);
    assert_eq!(clusters[1].commits[0].sha, "ccc");

    // Third cluster: ddd
    assert_eq!(clusters[2].commits.len(), 1);
    assert_eq!(clusters[2].commits[0].sha, "ddd");
}

// ---------------------------------------------------------------------------
// Test 10: test_blob_store
// ---------------------------------------------------------------------------

#[test]
fn test_blob_store() {
    let dir = tempdir().unwrap();
    let store = BlobStore::new(dir.path()).unwrap();

    let data = b"hello blob world";
    let hash = store.store_blob(data).unwrap();
    assert!(!hash.is_empty());

    // Retrieve by hash
    let retrieved = store.get_blob(&hash).unwrap();
    assert_eq!(retrieved, data);

    // Store same data again -- should produce the same hash (content-addressable)
    let hash2 = store.store_blob(data).unwrap();
    assert_eq!(
        hash, hash2,
        "storing identical data should yield the same hash"
    );
}

// ---------------------------------------------------------------------------
// Test 11: test_conversation_storage
// ---------------------------------------------------------------------------

#[test]
fn test_conversation_storage() {
    let dir = tempdir().unwrap();
    let db = create_db(dir.path());
    db.init_schema().unwrap();

    let intent_id = intent::create_intent(&db, "conversation test").unwrap();
    let session_id = SessionManager::start_session(&db, &intent_id).unwrap();

    // Insert conversation records directly via db.conn
    let messages = vec![
        ("conv-1", "Hello, I need help with auth"),
        ("conv-2", "Sure, let me look at the code"),
        ("conv-3", "I found the issue in session.rs"),
    ];

    for (id, message) in &messages {
        db.conn
            .execute(
                "INSERT INTO conversations (id, session_id, message, created_at) VALUES (?1, ?2, ?3, datetime('now'))",
                rusqlite::params![id, session_id, message],
            )
            .unwrap();
    }

    // Query them back
    let mut stmt = db
        .conn
        .prepare(
            "SELECT id, session_id, message FROM conversations WHERE session_id = ?1 ORDER BY id",
        )
        .unwrap();
    let rows: Vec<(String, String, String)> = stmt
        .query_map(rusqlite::params![session_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, "conv-1");
    assert_eq!(rows[0].2, "Hello, I need help with auth");
    assert_eq!(rows[1].2, "Sure, let me look at the code");
    assert_eq!(rows[2].2, "I found the issue in session.rs");

    // All rows should reference our session
    for row in &rows {
        assert_eq!(row.1, session_id);
    }
}

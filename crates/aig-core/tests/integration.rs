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
use aig_core::sync;

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

// ---------------------------------------------------------------------------
// Test 12: test_push_pull_notes_roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_push_pull_notes_roundtrip() {
    let dir = tempdir().unwrap();
    let repo = init_test_repo(dir.path());
    let db = create_db(dir.path());
    db.init_schema().unwrap();

    // Create an intent
    let intent_id = intent::create_intent(&db, "roundtrip test intent").unwrap();

    // Start a session
    let session_id = SessionManager::start_session(&db, &intent_id).unwrap();

    // Write a file and create a git commit
    std::fs::write(dir.path().join("roundtrip.txt"), "roundtrip content").unwrap();
    let sha = git_interop::create_commit(&repo, "add roundtrip.txt").unwrap();

    // Create a checkpoint
    let cp_id =
        CheckpointManager::create_checkpoint(&db, &session_id, "roundtrip checkpoint", &sha, &repo)
            .unwrap();
    assert!(!cp_id.is_empty());

    // Add a conversation record
    db.conn
        .execute(
            "INSERT INTO conversations (id, session_id, message, created_at) VALUES (?1, ?2, ?3, datetime('now'))",
            rusqlite::params!["conv-rt-1", session_id, "roundtrip conversation message"],
        )
        .unwrap();

    // Push notes
    let pushed = sync::push_notes(&db, &repo).unwrap();
    assert_eq!(pushed, 1, "should push exactly 1 note");

    // Verify git note exists on the commit
    let oid = git2::Oid::from_str(&sha).unwrap();
    let note = repo.find_note(Some("refs/notes/aig"), oid);
    assert!(note.is_ok(), "git note should exist on the commit");

    // Delete the DB and recreate with fresh schema
    let aig_dir = dir.path().join(".aig");
    let db_path = aig_dir.join("aig.db");
    drop(db);
    std::fs::remove_file(&db_path).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let db2 = Database { conn };
    db2.init_schema().unwrap();

    // Pull notes
    let pulled = sync::pull_notes(&db2, &repo).unwrap();
    assert_eq!(pulled, 1, "should pull exactly 1 note");

    // Verify the intent exists in the new DB
    let fetched_intent = intent::get_intent(&db2, &intent_id).unwrap();
    assert_eq!(fetched_intent.description, "roundtrip test intent");

    // Verify the checkpoint exists
    let cp_row: (String, String) = db2
        .conn
        .query_row(
            "SELECT id, git_commit_sha FROM checkpoints WHERE id = ?1",
            rusqlite::params![cp_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(cp_row.0, cp_id);
    assert_eq!(cp_row.1, sha);

    // Verify conversations were restored
    let conv_count: i64 = db2
        .conn
        .query_row(
            "SELECT COUNT(*) FROM conversations WHERE session_id = ?1",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(conv_count, 1, "conversation should be restored");
}

// ---------------------------------------------------------------------------
// Test 13: test_pull_notes_idempotent
// ---------------------------------------------------------------------------

#[test]
fn test_pull_notes_idempotent() {
    let dir = tempdir().unwrap();
    let repo = init_test_repo(dir.path());
    let db = create_db(dir.path());
    db.init_schema().unwrap();

    // Create intent, session, commit, checkpoint
    let intent_id = intent::create_intent(&db, "idempotent test").unwrap();
    let session_id = SessionManager::start_session(&db, &intent_id).unwrap();
    std::fs::write(dir.path().join("idem.txt"), "idempotent content").unwrap();
    let sha = git_interop::create_commit(&repo, "add idem.txt").unwrap();
    let _cp_id =
        CheckpointManager::create_checkpoint(&db, &session_id, "idem checkpoint", &sha, &repo)
            .unwrap();

    // Push notes
    sync::push_notes(&db, &repo).unwrap();

    // Fresh DB
    let aig_dir = dir.path().join(".aig");
    let db_path = aig_dir.join("aig.db");
    drop(db);
    std::fs::remove_file(&db_path).unwrap();
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let db2 = Database { conn };
    db2.init_schema().unwrap();

    // Pull twice
    let pulled1 = sync::pull_notes(&db2, &repo).unwrap();
    assert_eq!(pulled1, 1);
    let pulled2 = sync::pull_notes(&db2, &repo).unwrap();
    assert_eq!(pulled2, 1); // still iterates, but INSERT OR IGNORE prevents duplicates

    // Verify no duplicates
    let intent_count: i64 = db2
        .conn
        .query_row("SELECT COUNT(*) FROM intents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        intent_count, 1,
        "should have exactly 1 intent, not duplicates"
    );

    let cp_count: i64 = db2
        .conn
        .query_row("SELECT COUNT(*) FROM checkpoints", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        cp_count, 1,
        "should have exactly 1 checkpoint, not duplicates"
    );
}

// ---------------------------------------------------------------------------
// Test 14: test_incremental_import
// ---------------------------------------------------------------------------

#[test]
fn test_incremental_import() {
    let dir = tempdir().unwrap();
    let repo = init_test_repo(dir.path());

    // Create 3 commits
    for i in 1..=3 {
        let filename = format!("file{}.txt", i);
        std::fs::write(dir.path().join(&filename), format!("content {}", i)).unwrap();
        git_interop::create_commit(&repo, &format!("commit {}", i)).unwrap();
    }

    // Create the .aig DB
    let db = create_db(dir.path());
    db.init_schema().unwrap();

    // First import using the repo path
    // We can't call import_git_history directly because it creates its own Database::new().
    // Instead, test the idempotency logic by importing manually, then re-importing.

    // Get commits and cluster them
    let commits = git_interop::get_log(&repo, 100).unwrap();
    // Should have 4 commits: initial + 3 we created
    assert_eq!(commits.len(), 4);

    let clusters = cluster_commits(commits);

    // Manually import all clusters (simulating first import)
    for cluster in &clusters {
        let intent_id = intent::create_intent(&db, &cluster.inferred_intent).unwrap();
        for commit in &cluster.commits {
            let cp_id = format!("cp-{}", &commit.sha[..8]);
            let created_at = chrono::DateTime::from_timestamp(commit.timestamp, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
            db.conn
                .execute(
                    "INSERT INTO checkpoints (id, session_id, intent_id, git_commit_sha, message, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![cp_id, Option::<String>::None, intent_id, commit.sha, commit.message, created_at],
                )
                .unwrap();
        }
    }

    let intent_count_before: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM intents", [], |row| row.get(0))
        .unwrap();
    let cp_count_before: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM checkpoints", [], |row| row.get(0))
        .unwrap();

    // Create 2 more commits
    for i in 4..=5 {
        let filename = format!("file{}.txt", i);
        std::fs::write(dir.path().join(&filename), format!("content {}", i)).unwrap();
        git_interop::create_commit(&repo, &format!("commit {}", i)).unwrap();
    }

    // Get new commits and cluster them
    let all_commits = git_interop::get_log(&repo, 100).unwrap();
    assert_eq!(all_commits.len(), 6); // initial + 5

    let all_clusters = cluster_commits(all_commits);

    // Simulate incremental import: skip commits that already have checkpoints
    let mut new_intents = 0usize;
    let mut new_commits = 0usize;
    let mut skipped = 0usize;

    for cluster in &all_clusters {
        let mut new_in_cluster = Vec::new();
        for commit in &cluster.commits {
            let exists: bool = db
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM checkpoints WHERE git_commit_sha = ?1",
                    rusqlite::params![commit.sha],
                    |row| row.get(0),
                )
                .unwrap();
            if exists {
                skipped += 1;
            } else {
                new_in_cluster.push(commit);
            }
        }

        if new_in_cluster.is_empty() {
            continue;
        }

        let intent_id = intent::create_intent(&db, &cluster.inferred_intent).unwrap();
        for commit in &new_in_cluster {
            let cp_id = format!("cp-new-{}", &commit.sha[..8]);
            let created_at = chrono::DateTime::from_timestamp(commit.timestamp, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
            db.conn
                .execute(
                    "INSERT INTO checkpoints (id, session_id, intent_id, git_commit_sha, message, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![cp_id, Option::<String>::None, intent_id, commit.sha, commit.message, created_at],
                )
                .unwrap();
        }
        new_intents += 1;
        new_commits += new_in_cluster.len();
    }

    // The original 4 commits (initial + 3) should have been skipped
    assert!(
        skipped >= 4,
        "at least 4 commits should be skipped, got {}",
        skipped
    );

    // New commits should have been imported
    assert!(
        new_commits >= 2,
        "at least 2 new commits should be imported, got {}",
        new_commits
    );

    // Verify no duplicates: total checkpoints should be before + new
    let cp_count_after: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM checkpoints", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        cp_count_after,
        cp_count_before + new_commits as i64,
        "checkpoint count should increase by exactly the number of new commits"
    );

    let intent_count_after: i64 = db
        .conn
        .query_row("SELECT COUNT(*) FROM intents", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        intent_count_after,
        intent_count_before + new_intents as i64,
        "intent count should increase by exactly the number of new intents"
    );
}

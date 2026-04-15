use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::Connection;
use std::fs;
use std::io::Read;
use std::path::Path;

const BUNDLE_VERSION: u32 = 1;
const META_FILENAME: &str = "aig-bundle-meta.json";

pub fn export_bundle(output_path: &Path) -> Result<()> {
    let aig_dir = Path::new(".aig");
    let db_path = aig_dir.join("aig.db");
    let objects_dir = aig_dir.join("objects");

    if !db_path.exists() {
        bail!("no aig database found at .aig/aig.db");
    }

    // Vacuum the database into a temp file for a clean, self-contained copy
    let temp_db = output_path.with_extension("tmp.db");
    {
        let conn = Connection::open(&db_path).context("failed to open aig database")?;
        conn.execute_batch(&format!(
            "VACUUM INTO '{}';",
            temp_db.to_string_lossy().replace('\'', "''")
        ))
        .context("failed to vacuum database")?;
    }

    let file = fs::File::create(output_path)
        .with_context(|| format!("failed to create {}", output_path.display()))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut ar = tar::Builder::new(enc);

    // Write metadata
    let meta = serde_json::json!({
        "version": BUNDLE_VERSION,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "aig_version": env!("CARGO_PKG_VERSION"),
    });
    let meta_bytes = serde_json::to_vec_pretty(&meta)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(meta_bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    ar.append_data(&mut header, META_FILENAME, &meta_bytes[..])?;

    // Add the vacuumed database
    ar.append_path_with_name(&temp_db, "aig.db")
        .context("failed to add database to bundle")?;

    // Add objects directory
    let mut object_count = 0u64;
    if objects_dir.exists() {
        for entry in walkdir(&objects_dir)? {
            let rel = entry
                .strip_prefix(aig_dir)
                .context("failed to compute relative path")?;
            ar.append_path_with_name(&entry, rel)
                .with_context(|| format!("failed to add {}", entry.display()))?;
            object_count += 1;
        }
    }

    ar.finish()?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_db);

    let bundle_size = fs::metadata(output_path)?.len();
    println!(
        "Exported aig bundle to {} ({}, {} objects)",
        output_path.display(),
        format_size(bundle_size),
        object_count,
    );

    Ok(())
}

pub fn import_bundle(bundle_path: &Path, force: bool) -> Result<()> {
    let aig_dir = Path::new(".aig");

    if aig_dir.exists() && !force {
        bail!(".aig directory already exists. Use --force to overwrite, or remove it first.");
    }

    let file = fs::File::open(bundle_path)
        .with_context(|| format!("failed to open {}", bundle_path.display()))?;
    let dec = GzDecoder::new(file);
    let mut ar = tar::Archive::new(dec);

    // Validate bundle by checking for metadata entry
    let entries = ar
        .entries()
        .context("failed to read bundle (is this a valid .tar.gz file?)")?;
    let mut found_meta = false;
    let mut found_db = false;
    let mut object_count = 0u64;

    // If overwriting, remove existing .aig first
    if aig_dir.exists() && force {
        fs::remove_dir_all(aig_dir).context("failed to remove existing .aig directory")?;
    }
    fs::create_dir_all(aig_dir)?;

    for entry in entries {
        let mut entry = entry.context("failed to read bundle entry")?;
        let path = entry.path()?.to_path_buf();
        let path_str = path.to_string_lossy();

        if path_str == META_FILENAME {
            // Validate metadata
            let mut content = String::new();
            entry
                .read_to_string(&mut content)
                .context("failed to read bundle metadata")?;
            let meta: serde_json::Value =
                serde_json::from_str(&content).context("invalid bundle metadata")?;
            let version = meta["version"].as_u64().unwrap_or(0) as u32;
            if version > BUNDLE_VERSION {
                bail!(
                    "bundle version {version} is not supported by this version of aig (supports up to {BUNDLE_VERSION})"
                );
            }
            found_meta = true;
        } else if path_str == "aig.db" {
            entry
                .unpack(aig_dir.join("aig.db"))
                .context("failed to extract database")?;
            found_db = true;
        } else if path_str.starts_with("objects/") || path_str.starts_with("objects\\") {
            let dest = aig_dir.join(&*path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            entry
                .unpack(&dest)
                .with_context(|| format!("failed to extract {}", path.display()))?;
            object_count += 1;
        }
    }

    if !found_meta {
        // Clean up on invalid bundle
        let _ = fs::remove_dir_all(aig_dir);
        bail!("not a valid aig bundle (missing {META_FILENAME})");
    }
    if !found_db {
        let _ = fs::remove_dir_all(aig_dir);
        bail!("not a valid aig bundle (missing aig.db)");
    }

    // Verify database integrity
    let db_path = aig_dir.join("aig.db");
    let conn = Connection::open(&db_path).context("failed to open imported database")?;
    let integrity: String = conn.query_row("PRAGMA integrity_check;", [], |row| row.get(0))?;
    if integrity != "ok" {
        bail!("imported database failed integrity check: {integrity}");
    }

    println!(
        "Imported aig bundle from {} ({} objects)",
        bundle_path.display(),
        object_count,
    );

    Ok(())
}

/// Recursively collect all files under a directory.
fn walkdir(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(walkdir(&path)?);
        } else {
            files.push(path);
        }
    }
    Ok(files)
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    // Tests share process-wide cwd, so serialize them
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_round_trip() -> Result<()> {
        let _lock = CWD_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        let repo_dir = tmp.path().join("repo");
        fs::create_dir_all(repo_dir.join(".aig/objects/ab"))?;

        // Create a test database
        let db_path = repo_dir.join(".aig/aig.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "CREATE TABLE test (id TEXT, value TEXT);
             INSERT INTO test VALUES ('1', 'hello');",
        )?;
        drop(conn);

        // Create a test object
        fs::write(repo_dir.join(".aig/objects/ab/cdef1234"), b"blob data")?;

        // Export
        let bundle_path = tmp.path().join("test.aig-bundle.tar.gz");
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&repo_dir)?;

        export_bundle(&bundle_path)?;
        assert!(bundle_path.exists());

        // Delete .aig and re-import
        fs::remove_dir_all(".aig")?;
        assert!(!Path::new(".aig").exists());

        import_bundle(&bundle_path, false)?;

        // Verify database
        let conn = Connection::open(".aig/aig.db")?;
        let value: String =
            conn.query_row("SELECT value FROM test WHERE id = '1'", [], |row| row.get(0))?;
        assert_eq!(value, "hello");

        // Verify object
        let blob = fs::read(".aig/objects/ab/cdef1234")?;
        assert_eq!(blob, b"blob data");

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    #[test]
    fn test_overwrite_protection() -> Result<()> {
        let _lock = CWD_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        let repo_dir = tmp.path().join("repo");
        fs::create_dir_all(repo_dir.join(".aig"))?;

        // Create a minimal valid bundle first
        let bundle_dir = tmp.path().join("src");
        fs::create_dir_all(bundle_dir.join(".aig"))?;
        let conn = Connection::open(bundle_dir.join(".aig/aig.db"))?;
        conn.execute_batch("CREATE TABLE t (id TEXT);")?;
        drop(conn);

        let bundle_path = tmp.path().join("test.tar.gz");
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&bundle_dir)?;
        export_bundle(&bundle_path)?;
        std::env::set_current_dir(&repo_dir)?;

        // Import without force should fail
        let result = import_bundle(&bundle_path, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already exists"));

        // Import with force should succeed
        import_bundle(&bundle_path, true)?;

        std::env::set_current_dir(original_dir)?;
        Ok(())
    }

    #[test]
    fn test_invalid_bundle() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let bad_file = tmp.path().join("not-a-bundle.tar.gz");
        fs::write(&bad_file, b"this is not a tar.gz file")?;

        let result = import_bundle(&bad_file, false);
        assert!(result.is_err());
        Ok(())
    }
}

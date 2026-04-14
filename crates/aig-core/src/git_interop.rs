use anyhow::{Context, Result};
use git2::{Repository, Signature};

#[derive(Debug)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
}

pub fn open_repo(path: &str) -> Result<Repository> {
    Repository::open(path).with_context(|| format!("failed to open git repo at {path}"))
}

pub fn create_commit(repo: &Repository, message: &str) -> Result<String> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    let sig = repo
        .signature()
        .unwrap_or_else(|_| Signature::now("aig", "aig@localhost").unwrap());

    let parent_commit = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

    let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    Ok(oid.to_string())
}

pub fn get_log(repo: &Repository, limit: usize) -> Result<Vec<CommitInfo>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    for (i, oid) in revwalk.enumerate() {
        if i >= limit {
            break;
        }
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        commits.push(CommitInfo {
            sha: oid.to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            timestamp: commit.time().seconds(),
        });
    }
    Ok(commits)
}

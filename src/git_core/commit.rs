use chrono::{DateTime, Utc, TimeZone};
use git2::Repository;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommitInfo {
    pub sha: String, pub short_sha: String, pub message: String,
    pub author_name: String, pub author_email: String,
    pub timestamp: DateTime<Utc>, pub formatted_time: String,
}

#[derive(Debug, Serialize)]
pub struct CommitDetail {
    pub sha: String, pub short_sha: String, pub message: String,
    pub message_full: String, pub author_name: String, pub author_email: String,
    pub timestamp: DateTime<Utc>, pub parent_shas: Vec<String>,
    pub files_changed: Vec<FileChange>,
}

#[derive(Debug, Serialize)]
pub struct FileChange { pub filename: String, pub additions: usize, pub deletions: usize }

pub fn list_commits(repo: &Repository, rev: &str, page: u32, per_page: u32) -> Result<(Vec<CommitInfo>, u64), git2::Error> {
    let obj = repo.revparse_single(rev)?;
    let commit = obj.peel_to_commit()?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push(commit.id())?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    let total = revwalk.count() as u64;
    let skip = ((page - 1) * per_page) as usize;

    let mut revwalk = repo.revwalk()?;
    revwalk.push(commit.id())?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let commits: Vec<CommitInfo> = revwalk.skip(skip).take(per_page as usize).filter_map(|oid| {
        let oid = oid.ok()?;
        let commit = repo.find_commit(oid).ok()?;
        let time = commit.time();
        let dt = Utc.timestamp_opt(time.seconds(), 0).single()?;
        let author = commit.author();
        let name = author.name().unwrap_or("Unknown").to_string();
        let email = author.email().unwrap_or("").to_string();
        drop(author);
        let msg = commit.message().unwrap_or("").lines().next().unwrap_or("").to_string();
        Some(CommitInfo {
            sha: oid.to_string(), short_sha: oid.to_string()[..8].to_string(),
            message: msg, author_name: name, author_email: email,
            formatted_time: format_time(dt), timestamp: dt,
        })
    }).collect();
    Ok((commits, total))
}

pub fn get_commit_detail(repo: &Repository, sha: &str) -> Result<CommitDetail, git2::Error> {
    let obj = repo.revparse_single(sha)?;
    let commit = obj.peel_to_commit()?;
    let time = commit.time();
    let dt = Utc.timestamp_opt(time.seconds(), 0).single().unwrap_or_else(|| Utc::now());
    let author = commit.author();
    let author_name = author.name().unwrap_or("Unknown").to_string();
    let author_email = author.email().unwrap_or("").to_string();
    drop(author);
    let parent_shas: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();
    let tree = commit.tree()?;
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
    let mut files_changed = Vec::new();
    if let Some(ref parent_tree) = parent_tree {
        let diff = repo.diff_tree_to_tree(Some(parent_tree), Some(&tree), None)?;
        let stats = diff.stats()?;
        let count = stats.files_changed();
        for i in 0..count {
            let name = diff.deltas().nth(i).map(|d| {
                d.new_file().path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
            }).unwrap_or_default();
            files_changed.push(FileChange {
                filename: name,
                additions: stats.insertions() as usize,
                deletions: stats.deletions() as usize,
            });
        }
    }
    Ok(CommitDetail {
        sha: commit.id().to_string(), short_sha: commit.id().to_string()[..8].to_string(),
        message: commit.message().unwrap_or("").lines().next().unwrap_or("").to_string(),
        message_full: commit.message().unwrap_or("").to_string(),
        author_name, author_email, timestamp: dt,
        parent_shas, files_changed,
    })
}

pub fn get_commit_count(repo: &Repository, rev: &str) -> Result<u64, git2::Error> {
    let obj = repo.revparse_single(rev)?;
    let commit = obj.peel_to_commit()?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push(commit.id())?;
    Ok(revwalk.count() as u64)
}

fn format_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now - dt;
    if diff.num_hours() < 24 { format!("{} hours ago", diff.num_hours()) }
    else if diff.num_days() < 30 { format!("{} days ago", diff.num_days()) }
    else { dt.format("%Y-%m-%d").to_string() }
}

use git2::Repository;
use std::path::{Path, PathBuf};
use std::fs;

pub fn repo_path(data_dir: &str, owner_id: &str, repo_name: &str) -> PathBuf {
    PathBuf::from(data_dir)
        .join("repositories")
        .join(owner_id)
        .join(format!("{}.git", repo_name))
}

pub fn init_bare(path: &Path) -> Result<Repository, git2::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    Repository::init_bare(path)
}

pub fn open_bare(path: &Path) -> Result<Repository, git2::Error> {
    Repository::open_bare(path)
}

pub fn default_branch(repo: &Repository) -> String {
    match repo.head() {
        Ok(head) => head.shorthand().unwrap_or("main").to_string(),
        Err(_) => "main".to_string(),
    }
}

pub fn branches(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    let mut branch_names = Vec::new();
    for branch in repo.branches(Some(git2::BranchType::Local))? {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            branch_names.push(name.to_string());
        }
    }
    Ok(branch_names)
}

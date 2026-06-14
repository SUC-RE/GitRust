use git2::Repository;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub entry_type: String,
    pub size: Option<i64>,
}

pub fn list_tree(repo: &Repository, rev: &str, path: &str) -> Result<Vec<FileEntry>, git2::Error> {
    let obj = repo.revparse_single(rev)?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;

    let target = if path.is_empty() || path == "/" {
        tree
    } else {
        let entry = tree.get_path(Path::new(path))?;
        repo.find_tree(entry.id())?
    };

    let mut entries = Vec::new();
    for entry in target.iter() {
        let name = entry.name().unwrap_or("").to_string();
        let entry_path = if path.is_empty() || path == "/" {
            name.clone()
        } else {
            format!("{}/{}", path.trim_end_matches('/'), name)
        };
        let entry_type = match entry.kind() {
            Some(git2::ObjectType::Tree) => "directory".to_string(),
            Some(git2::ObjectType::Blob) => "file".to_string(),
            _ => "unknown".to_string(),
        };
        let size = if entry.kind() == Some(git2::ObjectType::Blob) {
            repo.find_blob(entry.id()).ok().map(|b| b.size() as i64)
        } else {
            None
        };
        entries.push(FileEntry { name, path: entry_path, entry_type, size });
    }

    entries.sort_by(|a, b| {
        if a.entry_type == b.entry_type {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else if a.entry_type == "directory" {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    Ok(entries)
}

pub fn find_readme(repo: &Repository, rev: &str) -> Option<(String, Vec<u8>)> {
    let tree = list_tree(repo, rev, "").ok()?;
    for entry in &tree {
        let name_lower = entry.name.to_lowercase();
        if name_lower == "readme.md" || name_lower == "readme.markdown" || name_lower == "readme" || name_lower == "readme.txt" {
            let content = crate::git_core::blob::read_blob(repo, rev, &entry.path).ok()?;
            return Some((entry.name.clone(), content));
        }
    }
    None
}

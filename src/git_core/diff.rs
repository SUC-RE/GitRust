use git2::{DiffOptions, Patch, Repository};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DiffLine { pub line_type: String, pub old_lineno: Option<u32>, pub new_lineno: Option<u32>, pub content: String }

#[derive(Debug, Serialize)]
pub struct DiffFile { pub old_path: String, pub new_path: String, pub status: String, pub lines: Vec<DiffLine> }

pub fn commit_diff(repo: &Repository, sha: &str) -> Result<Vec<DiffFile>, git2::Error> {
    let obj = repo.revparse_single(sha)?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut opts))?;
    build_diff_files(&diff)
}

pub fn compare_branches(repo: &Repository, from: &str, to: &str) -> Result<Vec<DiffFile>, git2::Error> {
    let from_tree = repo.revparse_single(from)?.peel_to_commit()?.tree()?;
    let to_tree = repo.revparse_single(to)?.peel_to_commit()?.tree()?;
    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), Some(&mut opts))?;
    build_diff_files(&diff)
}

fn build_diff_files(diff: &git2::Diff) -> Result<Vec<DiffFile>, git2::Error> {
    let mut files = Vec::new();
    let deltas: Vec<_> = diff.deltas().collect();
    for (i, delta) in deltas.iter().enumerate() {
        let status = match delta.status() {
            git2::Delta::Added => "added", git2::Delta::Deleted => "deleted",
            git2::Delta::Modified => "modified", git2::Delta::Renamed => "renamed",
            _ => "unknown",
        };
        let mut lines = Vec::new();
        if let Some(patch) = Patch::from_diff(diff, i)? {
            let num_hunks = patch.num_hunks();
            for hunk_i in 0..num_hunks {
                
                let hunk_line_count = patch.num_lines_in_hunk(hunk_i)?;
                for line_i in 0..hunk_line_count {
                    let line = patch.line_in_hunk(hunk_i, line_i)?;
                    let lt = match line.origin() {
                        '+' => "add", '-' => "del", _ => "context"
                    };
                    lines.push(DiffLine {
                        line_type: lt.to_string(),
                        old_lineno: Some(line.old_lineno().unwrap_or(0) as u32),
                        new_lineno: Some(line.new_lineno().unwrap_or(0) as u32),
                        content: String::from_utf8_lossy(line.content()).to_string(),
                    });
                }
            }
        }
        files.push(DiffFile {
            old_path: delta.old_file().path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            new_path: delta.new_file().path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            status: status.to_string(), lines,
        });
    }
    Ok(files)
}

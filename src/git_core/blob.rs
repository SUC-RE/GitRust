use git2::Repository;
use std::path::Path;

pub fn read_blob(repo: &Repository, rev: &str, path: &str) -> Result<Vec<u8>, git2::Error> {
    let obj = repo.revparse_single(rev)?;
    let commit = obj.peel_to_commit()?;
    let tree = commit.tree()?;
    let entry = tree.get_path(Path::new(path))?;
    let blob = repo.find_blob(entry.id())?;
    Ok(blob.content().to_vec())
}

pub fn detect_language(path: &str) -> Option<String> {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "rs" => Some("rust".into()), "py" => Some("python".into()),
        "js" => Some("javascript".into()), "ts" => Some("typescript".into()),
        "go" => Some("go".into()), "java" => Some("java".into()),
        "c" | "h" => Some("c".into()), "cpp" | "hpp" => Some("cpp".into()),
        "html" | "htm" => Some("html".into()), "css" => Some("css".into()),
        "json" => Some("json".into()), "xml" => Some("xml".into()),
        "yaml" | "yml" => Some("yaml".into()), "toml" => Some("toml".into()),
        "md" | "markdown" => Some("markdown".into()),
        "sh" | "bash" => Some("bash".into()), "sql" => Some("sql".into()),
        _ => None,
    }
}

pub fn is_binary(data: &[u8]) -> bool { data.iter().take(8000).any(|&b| b == 0) }

pub fn format_size(bytes: i64) -> String {
    if bytes < 1024 { format!("{} B", bytes) }
    else if bytes < 1048576 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else if bytes < 1073741824 { format!("{:.1} MB", bytes as f64 / 1048576.0) }
    else { format!("{:.1} GB", bytes as f64 / 1073741824.0) }
}

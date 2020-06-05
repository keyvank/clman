use git2::Repository;
use std::fs;
use std::path::Path;

pub fn clone(repo: &str) {
    let url = "https://github.com/".to_string() + repo;
    let root = Path::new("packages");
    let repo: Vec<&str> = repo.split("/").collect();
    assert_eq!(repo.len(), 2);
    let repo_path = root.join(repo[1]);
    if Path::exists(&repo_path) {
        fs::remove_dir_all(repo_path).unwrap();
    }
    fs::create_dir_all(root).unwrap();
    Repository::clone(&url, root.join(repo[1])).unwrap();
}

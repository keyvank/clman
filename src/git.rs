use git2::Repository;
use std::fs;
use std::path::Path;

pub fn clone(root: &Path, repo: String) {
    let url = "https://github.com/".to_string() + &repo[..];
    let repo = repo.split("/").collect::<Vec<_>>();
    assert_eq!(repo.len(), 2);
    let repo_name = repo[1];
    let repo_path = root.join(repo_name);
    if Path::exists(&repo_path) {
        fs::remove_dir_all(repo_path).unwrap();
    }
    fs::create_dir_all(root).unwrap();
    Repository::clone(&url, root.join(repo_name)).unwrap();
}

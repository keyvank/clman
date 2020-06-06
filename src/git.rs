use crate::utils;
use git2::Repository;
use std::fs;
use std::path::Path;

pub fn clone(root: &Path, repo: String) {
    let url = "https://github.com/".to_string() + &repo[..];
    let repo_path = root.join(utils::repo_name(&repo));
    if Path::exists(&repo_path) {
        fs::remove_dir_all(repo_path).unwrap();
    }
    fs::create_dir_all(root).unwrap();
    Repository::clone(&url, root.join(utils::repo_name(&repo))).unwrap();
}

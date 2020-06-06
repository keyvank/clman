use crate::conf::Dependency;
use git2::Repository;
use std::fs;
use std::path::Path;

pub fn clone(root: &Path, dep: &Dependency) {
    let url = "https://github.com/".to_string() + &dep.git[..];
    let repo_path = root.join(dep.name());
    if Path::exists(&repo_path) {
        fs::remove_dir_all(repo_path).unwrap();
    }
    fs::create_dir_all(root).unwrap();
    Repository::clone(&url, root.join(dep.name())).unwrap();
}

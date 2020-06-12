use crate::{error, utils};
use git2::Repository;
use std::fs;
use std::path::Path;

pub fn clone(root: &Path, repo: String, force: bool) -> error::ClmanResult<()> {
    let url = "https://github.com/".to_string() + &repo[..];
    let repo_path = root.join(utils::repo_name(&repo));
    if Path::exists(&repo_path) {
        if force {
            fs::remove_dir_all(repo_path)?;
        } else {
            return Ok(());
        }
    }
    println!("Fetching {}...", repo);
    fs::create_dir_all(root)?;
    Repository::clone(&url, root.join(utils::repo_name(&repo)))?;
    Ok(())
}

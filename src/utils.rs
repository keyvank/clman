use crate::error;
use std::process::Command;

pub fn repo_name(repo: &String) -> String {
    let repo = repo.split("/").collect::<Vec<_>>();
    assert_eq!(repo.len(), 2);
    repo[1].to_string()
}

pub fn get_output(cmd: &String) -> error::ClmanResult<String> {
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    if output.stderr.len() != 0 {
        Err(error::ClmanError::Command {
            stderr: std::str::from_utf8(&output.stderr[..]).unwrap().to_string(),
        })
    } else {
        Ok(std::str::from_utf8(&output.stdout[..]).unwrap().to_string())
    }
}

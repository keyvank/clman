use std::process::Command;

pub fn repo_name(repo: &String) -> String {
    let repo = repo.split("/").collect::<Vec<_>>();
    assert_eq!(repo.len(), 2);
    repo[1].to_string()
}

pub fn get_output(cmd: &String) -> String {
    std::str::from_utf8(
        &Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("failed to execute process")
            .stdout[..],
    )
    .unwrap()
    .to_string()
}

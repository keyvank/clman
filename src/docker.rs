use std::path::Path;
use std::process::Command;

pub fn gen(root: &Path, dockerfile: String, args: String) -> String {
    std::str::from_utf8(
        &Command::new("sh")
            .arg("-c")
            .arg(format!(
                "sudo docker run --rm $(sudo docker build -q -f {} {}) {}",
                root.join(dockerfile).to_str().unwrap().to_string(),
                root.to_str().unwrap().to_string(),
                args
            ))
            .output()
            .expect("failed to execute process")
            .stdout[..],
    )
    .unwrap()
    .to_string()
}

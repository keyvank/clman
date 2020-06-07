use crate::utils;
use std::path::Path;
use std::process::Command;

pub fn gen(root: &Path, dockerfile: String, args: String) -> String {
    let params = format!(
        "-f {} {}",
        root.join(dockerfile.clone()).to_str().unwrap().to_string(),
        root.to_str().unwrap().to_string()
    );
    Command::new("sh")
        .arg("-c")
        .arg(format!("sudo docker build {}", params))
        .spawn()
        .expect("failed to build docker image");

    utils::get_output(
        &format!(
            "sudo docker run --rm $(sudo docker build -q {}) {}",
            params, args
        )
        .to_string(),
    )
}

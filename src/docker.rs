use std::process::Command;

pub fn gen(dockerfile: String, args: String) -> String {
    println!("Running");
    std::str::from_utf8(
        &Command::new("sh")
            .arg("-c")
            .arg(format!(
                "sudo docker run --rm $(sudo docker build -q -f {} .) {}",
                dockerfile, args
            ))
            .output()
            .expect("failed to execute process")
            .stdout[..],
    )
    .unwrap()
    .to_string()
}

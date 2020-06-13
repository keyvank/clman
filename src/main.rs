extern crate clap;
extern crate dirs;
extern crate git2;
extern crate ocl;
extern crate sha2;

mod cl;
mod conf;
mod docker;
mod error;
mod git;
mod parse;
mod utils;

use clap::{App, Arg, SubCommand};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

pub fn cache_path() -> error::ClmanResult<PathBuf> {
    let path = dirs::home_dir().unwrap().join(".clman");
    if !Path::exists(&path) {
        fs::create_dir(&path)?;
    }
    Ok(path)
}

pub fn checksum(root: &Path, root_args: String) -> error::ClmanResult<String> {
    let conf = conf::read_config(root)?;
    let mut hasher = Sha256::new();
    hasher.input(root_args.as_bytes());

    for (name, src) in conf.src {
        hasher.input(name.as_bytes());
        match src {
            conf::Source::File { path } => {
                hasher.input(path.as_bytes());
                hasher.input(fs::read(root.join(path))?);
            }
            conf::Source::Dockerfile { dockerfile, args } => {
                hasher.input(dockerfile.as_bytes());
                hasher.input(fs::read(root.join(dockerfile))?);
                if let Some(args) = args {
                    hasher.input(args.as_bytes());
                }
            }
            conf::Source::Script { script, args } => {
                hasher.input(script.as_bytes());
                hasher.input(fs::read(root.join(script))?);
                if let Some(args) = args {
                    hasher.input(args.as_bytes());
                }
            }
            conf::Source::Package { git, args } => {
                hasher.input(git.as_bytes());
                if let Some(args) = args {
                    hasher.input(args.as_bytes());
                }
            }
        }
    }

    let mut s = String::new();
    for &byte in hasher.result()[..].iter() {
        write!(&mut s, "{:x}", byte).unwrap();
    }

    Ok(s)
}

pub fn clean(_root: &Path) {}

pub fn new(name: &str) -> error::ClmanResult<()> {
    let root = Path::new(name);
    let src_root = root.join("src");
    fs::create_dir(root.clone())?;
    fs::create_dir(src_root.clone())?;
    conf::write_config(root, conf::default())?;
    fs::write(src_root.join("main.cl"), include_str!("cl/main.cl"))?;
    fs::write(root.join(".gitignore"), "/packages\n")?;
    Ok(())
}

pub fn source(root: &Path, root_args: String) -> error::ClmanResult<String> {
    fetch(root, false)?;

    let cache_path = cache_path()?.join(checksum(root, root_args.clone())? + ".cl");

    if Path::exists(&cache_path) {
        return Ok(fs::read_to_string(cache_path)?);
    }

    let root_args = root_args.split(" ").collect::<Vec<_>>();
    let conf = conf::read_config(root)?;
    let mut ret = String::new();
    for (name, src) in conf.src {
        ret.push_str(
            &match src {
                conf::Source::File { path } => fs::read_to_string(&root.join(Path::new(&path)))?,
                conf::Source::Dockerfile { dockerfile, args } => {
                    println!("Generating {}...", name);
                    let mut subs = args.unwrap_or(String::new());
                    for i in 0..root_args.len() {
                        subs = subs.replace(&format!("${}", i + 1), root_args[i]);
                    }
                    docker::gen(root, dockerfile, subs)?
                }
                conf::Source::Script { script, args } => {
                    println!("Generating {}...", name);
                    let mut subs = args.unwrap_or(String::new());
                    for i in 0..root_args.len() {
                        subs = subs.replace(&format!("${}", i + 1), root_args[i]);
                    }
                    utils::get_output(
                        &(root.join(script).to_str().unwrap().to_string() + " " + &subs[..]),
                    )?
                }
                conf::Source::Package { git, args } => source(
                    &root.join("packages").join(utils::repo_name(&git)),
                    args.unwrap_or(String::new()),
                )?,
            }[..],
        );
    }

    fs::write(cache_path, ret.clone())?;

    Ok(ret)
}

pub fn fetch(root: &Path, force: bool) -> error::ClmanResult<()> {
    let conf = conf::read_config(root)?;
    for (_, source) in conf.src {
        if let conf::Source::Package { git, args: _ } = source {
            git::clone(&root.join("packages"), git, force)?;
        }
    }
    Ok(())
}

pub fn run(root: &Path, root_args: String) -> error::ClmanResult<()> {
    let conf = conf::read_config(root)?;
    let src = source(root, root_args)?;
    let mut gpu = cl::GPU::new(src)?;
    let mut buffers = HashMap::<String, Box<dyn cl::GenericBuffer>>::new();
    for (name, buff) in conf.buffers.iter() {
        buffers.insert(name.clone(), gpu.create_buffer(buff.r#type, buff.count)?);
    }
    for job in conf.jobs.iter() {
        match job {
            conf::Job::Run { kernel, args } => {
                let mut a = Vec::new();
                for arg in args {
                    match arg {
                        conf::Arg::Buffer(name) => {
                            a.push(cl::KernelArgument::Buffer(buffers.remove(name).unwrap()));
                        }
                        conf::Arg::Number(v) => {
                            a.push(cl::KernelArgument::Uint(*v));
                        }
                    }
                }
                gpu.run_kernel(kernel.clone(), a)?;
            }
        }
    }
    Ok(())
}

fn main() {
    let matches = App::new("Clman")
        .version(conf::VERSION)
        .author(conf::AUTHORS)
        .about(conf::DESCRIPTION)
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a new project")
                .arg(
                    Arg::with_name("NAME")
                        .help("Project name")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("run").about("Run the project in current directory"))
        .subcommand(SubCommand::with_name("gen").about("Generate final OpenCL source code"))
        .subcommand(SubCommand::with_name("fetch").about("Fetch git dependencies"))
        .subcommand(SubCommand::with_name("clean").about("Clean cache"))
        .subcommand(SubCommand::with_name("list").about("List available functions"))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        let name = matches.value_of("NAME").unwrap();
        new(name).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("run") {
        run(Path::new("."), String::new()).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("gen") {
        println!("{}", source(Path::new("."), String::new()).unwrap());
    }

    if let Some(_matches) = matches.subcommand_matches("fetch") {
        fetch(Path::new("."), true).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("clean") {
        clean(Path::new("."));
    }

    if let Some(_matches) = matches.subcommand_matches("list") {
        for f in parse::list_functions(source(Path::new("."), String::new()).unwrap()) {
            println!("{}", f);
        }
    }
}

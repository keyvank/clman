extern crate clap;
extern crate git2;
extern crate ocl;
mod cl;
mod conf;
mod docker;
mod git;

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use clap::{App, Arg, SubCommand};

pub fn new(name: &str) -> conf::ConfigResult<()> {
    let root = Path::new(name);
    let src_root = root.join("src");
    fs::create_dir(root.clone())?;
    fs::create_dir(src_root.clone())?;
    conf::write_config(root, conf::default())?;
    fs::write(src_root.join("main.cl"), include_str!("cl/main.cl"))?;
    fs::write(root.join(".gitignore"), "/packages\n")?;
    Ok(())
}

pub fn source(root: &Path, root_args: String) -> conf::ConfigResult<String> {
    let root_args = root_args.split(" ").collect::<Vec<_>>();
    let conf = conf::read_config(root).unwrap();
    let mut ret = String::new();
    if let Some(deps) = conf.deps {
        for (_, dep) in deps {
            ret.push_str(
                &source(
                    &root.join("packages").join(dep.name()),
                    dep.args.to_string(),
                )?[..],
            );
        }
    }
    if let Some(src) = conf.src {
        for (_, source) in src {
            ret.push_str(
                &match source {
                    conf::Source::File { path } => {
                        fs::read_to_string(&root.join(Path::new(&path)))?
                    }
                    conf::Source::Dockerfile { dockerfile, args } => {
                        let mut subs = args.unwrap_or(String::new());
                        for i in 0..root_args.len() {
                            subs = subs.replace(&format!("${}", i + 1), root_args[i]);
                        }
                        docker::gen(root, dockerfile, subs)
                    }
                }[..],
            );
        }
    }
    Ok(ret)
}

pub fn fetch(root: &Path) {
    let conf = conf::read_config(root).unwrap();
    if let Some(deps) = conf.deps {
        for (_, dep) in deps {
            git::clone(&root.join("packages"), &dep);
        }
    }
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
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        let name = matches.value_of("NAME").unwrap();
        new(name).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("run") {
        cl::run(source(Path::new("."), String::new()).unwrap()).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("gen") {
        println!("{}", source(Path::new("."), String::new()).unwrap());
    }

    if let Some(_matches) = matches.subcommand_matches("fetch") {
        fetch(Path::new("."));
    }
}

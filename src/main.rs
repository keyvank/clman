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

pub fn source(conf: conf::Config) -> conf::ConfigResult<String> {
    let mut ret = String::new();
    if let Some(src) = conf.src {
        for (_, source) in src {
            ret.push_str(
                &match source {
                    conf::Source::File { path } => fs::read_to_string(path)?,
                    conf::Source::Dockerfile { dockerfile, args } => {
                        docker::gen(dockerfile, args.unwrap_or(String::new()))
                    }
                }[..],
            );
        }
    }
    Ok(ret)
}

pub fn fetch(config: conf::Config) {
    if let Some(deps) = config.deps {
        for (_, dep) in deps {
            git::clone(&dep.git[..]);
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
        let conf = conf::read_config().unwrap();
        cl::run(source(conf).unwrap()).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("gen") {
        let conf = conf::read_config().unwrap();
        println!("{}", source(conf).unwrap());
    }

    if let Some(_matches) = matches.subcommand_matches("fetch") {
        let conf = conf::read_config().unwrap();
        fetch(conf);
    }
}

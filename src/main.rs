extern crate clap;
extern crate ocl;
extern crate yaml_rust;
mod conf;
use conf::*;
use std::fs;
use std::path::Path;

use clap::{App, Arg, SubCommand};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

pub fn new(name: &str) -> ConfigResult<()> {
    let root = Path::new(name);
    let src_root = root.join("src");
    fs::create_dir(root.clone())?;
    fs::create_dir(src_root.clone())?;
    write_config(
        &root,
        Config {
            version: VERSION.to_string(),
            deps: vec!["keyvank/cl-utils".to_string()],
            main: "main".to_string(),
        },
    )?;
    fs::write(src_root.join("main.cl"), include_str!("cl/main.cl"))?;
    Ok(())
}

fn main() {
    let matches = App::new("Clman")
        .version(VERSION)
        .author(AUTHORS)
        .about(DESCRIPTION)
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
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        let name = matches.value_of("NAME").unwrap();
        new(name).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("run") {
        let _conf = conf::read_config().unwrap();
    }
}

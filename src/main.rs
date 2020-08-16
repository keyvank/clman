extern crate clap;
extern crate dirs;
extern crate git2;
extern crate image;
extern crate rust_gpu_tools;
extern crate sha2;

mod cl;
mod conf;
mod docker;
mod error;
mod git;
mod parse;
mod utils;

use crate::conf::{Computable, Environment};
use clap::{App, Arg, SubCommand};
use image::ImageBuffer;
use itertools::*;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

fn save_image_float4(w: usize, h: usize, data: Vec<(f32, f32, f32, f32)>, path: &String) {
    assert_eq!(data.len(), w * h);
    let img = ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let pix = data[y as usize * w + x as usize];
        image::Rgba([
            (pix.0 * 255.0) as u8,
            (pix.1 * 255.0) as u8,
            (pix.2 * 255.0) as u8,
            (pix.3 * 255.0) as u8,
        ])
    });
    img.save(path).unwrap();
}

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

    for (k, v) in conf.define {
        hasher.input(k.as_bytes());
        hasher.input(v.0.as_bytes());
    }

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
                hasher.input(args.0.as_bytes());
            }
            conf::Source::Script { script, args } => {
                hasher.input(script.as_bytes());
                hasher.input(fs::read(root.join(script))?);
                hasher.input(args.0.as_bytes());
            }
            conf::Source::Package { git, args } => {
                hasher.input(git.as_bytes());
                hasher.input(args.0.as_bytes());
            }
        }
    }

    let mut s = String::new();
    for &byte in hasher.result()[..].iter() {
        write!(&mut s, "{:x}", byte).unwrap();
    }

    Ok(s)
}

pub fn clean(root: &Path) -> error::ClmanResult<()> {
    fs::remove_dir_all(cache_path()?)?;
    let packages_dir = root.join("packages");
    if Path::exists(&packages_dir) {
        fs::remove_dir_all(&packages_dir)?;
    }
    Ok(())
}

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

pub fn source(env: &Environment, root: &Path, root_args: String) -> error::ClmanResult<String> {
    fetch(root, false)?;

    let cache_path = cache_path()?.join(checksum(root, root_args.clone())? + ".cl");

    if Path::exists(&cache_path) {
        return Ok(fs::read_to_string(cache_path)?);
    }

    let conf = conf::read_config(root)?;
    let sub_env = {
        let mut env = Environment::new(Some(env.clone()));
        for (i, arg) in root_args.split(" ").enumerate() {
            env.set(i.to_string(), arg.into());
        }
        env
    };
    let mut ret = String::new();
    for (k, v) in conf.define.iter() {
        ret.push_str(&format!("#define {} ({})\n", k, v.compute(&sub_env)));
    }
    for (name, src) in conf.src {
        ret.push_str(
            &match src {
                conf::Source::File { path } => fs::read_to_string(&root.join(Path::new(&path)))?,
                conf::Source::Dockerfile { dockerfile, args } => {
                    println!("Generating {}...", name);
                    //let mut subs = args.unwrap_or(String::new());
                    //for i in 0..root_args.len() {
                    //    subs = subs.replace(&format!("${}", i + 1), root_args[i]);
                    //}
                    docker::gen(root, dockerfile, args.compute(&sub_env))?
                }
                conf::Source::Script { script, args } => {
                    println!("Generating {}...", name);
                    //let mut subs = args.unwrap_or(String::new());
                    //for i in 0..root_args.len() {
                    //    subs = subs.replace(&format!("${}", i + 1), root_args[i]);
                    //}
                    utils::get_output(
                        &(root.join(script).to_str().unwrap().to_string()
                            + " "
                            + &args.compute(&sub_env)),
                    )?
                }
                conf::Source::Package { git, args } => source(
                    env,
                    &root.join("packages").join(utils::repo_name(&git)),
                    args.compute(&sub_env),
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

pub fn run(env: &Environment, root: &Path, root_args: String) -> error::ClmanResult<()> {
    let conf = conf::read_config(root)?;
    let src = source(env, root, root_args.clone())?;

    let mut env = env.clone();
    for (i, arg) in root_args.split(" ").enumerate() {
        env.set(i.to_string(), arg.into());
    }

    let mut gpu = cl::GPU::new(src)?;
    for (name, buff) in conf.buffers.iter() {
        gpu.create_buffer(name.clone(), buff.r#type, buff.count.compute(&env))?;
    }
    for (_, job) in conf.jobs.iter() {
        match job {
            conf::Job::Run {
                run,
                args,
                global_work_size,
            } => {
                gpu.run_kernel(
                    &env,
                    run.compute(&env),
                    args.clone(),
                    global_work_size.compute(&env),
                )?;
            }
            conf::Job::Save { save, to } => match to {
                conf::Storage::Raw { path } => {
                    std::fs::write(&path.compute(&env), gpu.read_buffer(save.compute(&env))?)?;
                }
                conf::Storage::Image { x, y, path } => {
                    save_image_float4(
                        x.compute(&env),
                        y.compute(&env),
                        gpu.read_buffer(save.compute(&env))?,
                        &path.compute(&env),
                    );
                }
            },
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
        .subcommand(
            SubCommand::with_name("run")
                .arg(Arg::with_name("ARGS").min_values(1))
                .about("Run the project in current directory"),
        )
        .subcommand(SubCommand::with_name("gen").about("Generate final OpenCL source code"))
        .subcommand(SubCommand::with_name("fetch").about("Fetch git dependencies"))
        .subcommand(SubCommand::with_name("clean").about("Clean cache"))
        .subcommand(SubCommand::with_name("list").about("List available functions"))
        .get_matches();

    let env = conf::Environment::new(None);

    if let Some(matches) = matches.subcommand_matches("new") {
        let name = matches.value_of("NAME").unwrap();
        new(name).unwrap();
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        let args = matches
            .values_of("ARGS")
            .map(|mut vals| vals.join(" "))
            .unwrap_or_default();
        run(&env, Path::new("."), args.into()).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("gen") {
        println!("{}", source(&env, Path::new("."), String::new()).unwrap());
    }

    if let Some(_matches) = matches.subcommand_matches("fetch") {
        fetch(Path::new("."), true).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("clean") {
        clean(Path::new(".")).unwrap();
    }

    if let Some(_matches) = matches.subcommand_matches("list") {
        for f in parse::list_functions(source(&env, Path::new("."), String::new()).unwrap()) {
            println!("{}", f);
        }
    }
}

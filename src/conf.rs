use crate::error::ClmanResult;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Source {
    File {
        path: String,
    },
    Dockerfile {
        dockerfile: String,
        args: Option<String>,
    },
    Script {
        script: String,
        args: Option<String>,
    },
    Package {
        git: String,
        args: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arg {
    Buffer(String),
    Char(i8),
    Uchar(u8),
    Short(i16),
    Ushort(u16),
    Int(i32),
    Uint(u32),
    Long(i64),
    Ulong(u64),
    Float(f32),
    Double(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Job {
    Run {
        run: String,
        args: Vec<Arg>,
        global_work_size: usize,
    },
    Save {
        save: String,
        to: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BufferType {
    Char,
    Uchar,
    Short,
    Ushort,
    Int,
    Uint,
    Long,
    Ulong,
    Float,
    Double,
    Float4,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Buffer {
    pub r#type: BufferType,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub src: LinkedHashMap<String, Source>,
    #[serde(default)]
    pub buffers: LinkedHashMap<String, Buffer>,
    #[serde(default)]
    pub jobs: LinkedHashMap<String, Job>,
}

pub fn write_config(root: &Path, conf: Config) -> ClmanResult<()> {
    fs::write(root.join("clman.yaml"), serde_yaml::to_string(&conf)?)?;
    Ok(())
}

pub fn read_config(root: &Path) -> ClmanResult<Config> {
    let config: Config = serde_yaml::from_str(&fs::read_to_string(root.join("clman.yaml"))?)?;
    Ok(config)
}

pub fn default() -> Config {
    Config {
        version: VERSION.to_string(),
        src: {
            let mut src = LinkedHashMap::<String, Source>::new();
            src.insert(
                "ff.cl".to_string(),
                Source::Package {
                    git: "keyvank/ff-cl-gen".to_string(),
                    args: Some("Fp 52435875175126190479447740508185965837690552500527637822603658699938581184513".to_string()),
                },
            );
            src.insert(
                "main.cl".to_string(),
                Source::File {
                    path: "src/main.cl".to_string(),
                },
            );
            src
        },
        buffers: {
            let mut buffs = LinkedHashMap::<String, Buffer>::new();
            buffs.insert(
                "buff".to_string(),
                Buffer {
                    r#type: BufferType::Uint,
                    count: 1024,
                },
            );
            buffs
        },
        jobs: {
            let mut jobs = LinkedHashMap::<String, Job>::new();
            jobs.insert(
                "fill_buffer".to_string(),
                Job::Run {
                    run: "fill".to_string(),
                    args: vec![Arg::Buffer("buff".to_string()), Arg::Uint(3)],
                    global_work_size: 1024,
                },
            );
            jobs.insert(
                "calculate_sum".to_string(),
                Job::Run {
                    run: "sum".to_string(),
                    args: vec![Arg::Buffer("buff".to_string()), Arg::Uint(1024)],
                    global_work_size: 1,
                },
            );
            jobs
        },
    }
}

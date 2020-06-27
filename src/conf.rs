use crate::error::ClmanResult;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

pub struct Environment {
    pub parent: Option<Box<Environment>>,
    pub vars: HashMap<String, String>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            parent: None,
            vars: HashMap::new(),
        }
    }
    pub fn set(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }
    pub fn get(&self, key: String) -> Option<String> {
        match self.vars.get(&key) {
            Some(v) => Some(v.clone()),
            None => {
                if let Some(ref env) = self.parent {
                    env.get(key)
                } else {
                    None
                }
            }
        }
    }
}

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
#[serde(untagged)]
pub enum Value<T: Default + Clone> {
    Static(T),
    Dynamic(String),
}

impl<T: Default + Clone> Value<T> {
    pub fn compute(&self, env: &Environment) -> T {
        match self {
            Value::Static(v) => v.clone(),
            Value::Dynamic(_) => T::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arg {
    Buffer(String),
    Char(Value<i8>),
    Uchar(Value<u8>),
    Short(Value<i16>),
    Ushort(Value<u16>),
    Int(Value<i32>),
    Uint(Value<u32>),
    Long(Value<i64>),
    Ulong(Value<u64>),
    Float(Value<f32>),
    Double(Value<f64>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Storage {
    Raw {
        path: String,
    },
    Image {
        path: String,
        x: Value<usize>,
        y: Value<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Job {
    Run {
        run: String,
        args: Vec<Arg>,
        global_work_size: Value<usize>,
    },
    Save {
        save: String,
        to: Storage,
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
    pub count: Value<usize>,
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
                    count: Value::Static(1024),
                },
            );
            buffs.insert(
                "img".to_string(),
                Buffer {
                    r#type: BufferType::Float4,
                    count: Value::Static(256 * 256),
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
                    args: vec![Arg::Buffer("buff".to_string()), Arg::Uint(Value::Static(3))],
                    global_work_size: Value::Static(1024),
                },
            );
            jobs.insert(
                "calculate_sum".to_string(),
                Job::Run {
                    run: "sum".to_string(),
                    args: vec![
                        Arg::Buffer("buff".to_string()),
                        Arg::Uint(Value::Static(1024)),
                    ],
                    global_work_size: Value::Static(1),
                },
            );
            jobs.insert(
                "fill_img".to_string(),
                Job::Run {
                    run: "draw".to_string(),
                    args: vec![Arg::Buffer("img".to_string())],
                    global_work_size: Value::Static(256 * 256),
                },
            );
            jobs.insert(
                "save_img".to_string(),
                Job::Save {
                    save: "img".to_string(),
                    to: Storage::Image {
                        x: Value::Static(256),
                        y: Value::Static(256),
                        path: "img.bmp".to_string(),
                    },
                },
            );
            jobs
        },
    }
}

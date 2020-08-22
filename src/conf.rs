use crate::error::ClmanResult;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Clone)]
pub struct Environment {
    pub parent: Option<Box<Environment>>,
    pub vars: HashMap<String, String>,
}

impl Environment {
    pub fn new(parent: Option<Environment>) -> Self {
        Environment {
            parent: parent.map(|e| Box::new(e)),
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
    pub fn as_map(&self) -> HashMap<String, String> {
        let mut ret = self.vars.clone();
        let mut curr = self.parent.as_ref();
        while let Some(env) = curr {
            for (k, v) in env.vars.iter() {
                ret.entry(k.into()).or_insert(v.into());
            }
            curr = env.parent.as_ref();
        }
        ret
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Source {
    Code {
        code: ValueString,
    },
    File {
        path: String,
    },
    Dockerfile {
        dockerfile: String,
        args: ValueString,
    },
    Script {
        script: String,
        args: ValueString,
    },
    Package {
        git: String,
        args: ValueString,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueString(pub String);

impl From<&str> for ValueString {
    fn from(s: &str) -> Self {
        ValueString(String::from(s))
    }
}

pub trait Computable<T> {
    fn compute(&self, env: &Environment) -> T;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value<T: Default + Clone> {
    Static(T),
    Dynamic(ValueString),
}

impl Computable<String> for ValueString {
    fn compute(&self, env: &Environment) -> String {
        let mut ret = self.0.clone();
        for (k, v) in env.as_map().iter() {
            ret = ret.replace(&format!("${}", k), v);
        }
        if ret.starts_with("$((") && ret.ends_with("))") {
            ret = ret[3..ret.len() - 2].into();
            ret = mexprp::eval::<f64>(&ret)
                .unwrap()
                .unwrap_single()
                .to_string();
        }
        ret
    }
}

impl<T: Default + Clone + std::str::FromStr> Computable<T> for Value<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    fn compute(&self, env: &Environment) -> T {
        match self {
            Value::Static(v) => v.clone(),
            Value::Dynamic(s) => {
                let s = s.compute(env);
                T::from_str(&s).unwrap()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arg {
    Buffer(ValueString),
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
        path: ValueString,
    },
    Image {
        path: ValueString,
        x: Value<usize>,
        y: Value<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Job {
    Run {
        run: ValueString,
        args: Vec<Arg>,
        global_work_size: Value<usize>,
    },
    Save {
        save: ValueString,
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

impl BufferType {
    pub fn size_of(&self) -> usize {
        match self {
            Self::Char => 1,
            Self::Uchar => 1,
            Self::Short => 2,
            Self::Ushort => 2,
            Self::Int => 4,
            Self::Uint => 4,
            Self::Long => 8,
            Self::Ulong => 8,
            Self::Float => 4,
            Self::Double => 8,
            Self::Float4 => 16,
        }
    }
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
    pub define: LinkedHashMap<String, ValueString>,
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
        define: Default::default(),
        src: {
            let mut src = LinkedHashMap::<String, Source>::new();
            src.insert(
                "ff.cl".to_string(),
                Source::Package {
                    git: String::from("keyvank/ff-cl-gen"),
                    args: ValueString::from("Fp 52435875175126190479447740508185965837690552500527637822603658699938581184513"),
                },
            );
            src.insert(
                "main.cl".to_string(),
                Source::File {
                    path: String::from("src/main.cl"),
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
                    run: ValueString::from("fill"),
                    args: vec![
                        Arg::Buffer(ValueString::from("buff")),
                        Arg::Uint(Value::Static(3)),
                    ],
                    global_work_size: Value::Static(1024),
                },
            );
            jobs.insert(
                "calculate_sum".to_string(),
                Job::Run {
                    run: ValueString::from("sum"),
                    args: vec![
                        Arg::Buffer(ValueString::from("buff")),
                        Arg::Uint(Value::Static(1024)),
                    ],
                    global_work_size: Value::Static(1),
                },
            );
            jobs.insert(
                "fill_img".to_string(),
                Job::Run {
                    run: ValueString::from("draw"),
                    args: vec![Arg::Buffer(ValueString::from("img"))],
                    global_work_size: Value::Static(256 * 256),
                },
            );
            jobs.insert(
                "save_img".to_string(),
                Job::Save {
                    save: ValueString::from("img"),
                    to: Storage::Image {
                        x: Value::Static(256),
                        y: Value::Static(256),
                        path: ValueString::from("img.bmp"),
                    },
                },
            );
            jobs
        },
    }
}

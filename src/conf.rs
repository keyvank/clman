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
pub enum BufferType {
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "uint")]
    Uint,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "double")]
    Double,
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
                "src_array".to_string(),
                Buffer {
                    r#type: BufferType::Uint,
                    count: 1024,
                },
            );
            buffs.insert(
                "dst_array".to_string(),
                Buffer {
                    r#type: BufferType::Uint,
                    count: 1024,
                },
            );
            buffs
        },
    }
}

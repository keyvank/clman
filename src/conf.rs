use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Yaml Error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

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
    Package {
        git: String,
        args: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub src: LinkedHashMap<String, Source>,
}

pub fn write_config(root: &Path, conf: Config) -> ConfigResult<()> {
    fs::write(root.join("clman.yaml"), serde_yaml::to_string(&conf)?)?;
    Ok(())
}

pub fn read_config(root: &Path) -> ConfigResult<Config> {
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
    }
}

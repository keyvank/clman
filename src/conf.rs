use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub struct Dependency {
    pub git: String,
    pub args: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dynamic {
    pub dockerfile: String,
    pub args: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub deps: HashMap<String, Dependency>,
    pub dyns: HashMap<String, Dynamic>,
    pub main: String,
}

pub fn write_config(root: &Path, conf: Config) -> ConfigResult<()> {
    fs::write(root.join("clman.yaml"), serde_yaml::to_string(&conf)?)?;
    Ok(())
}

pub fn read_config() -> ConfigResult<Config> {
    let config: Config = serde_yaml::from_str(&fs::read_to_string("clman.yaml")?)?;
    Ok(config)
}

pub fn default() -> Config {
    Config {
        version: VERSION.to_string(),
        deps: {
            let mut deps = HashMap::<String, Dependency>::new();
            deps.insert(
                "ff".to_string(),
                Dependency {
                    git: "keyvank/ff-cl-gen".to_string(),
                    args: "Fp 52435875175126190479447740508185965837690552500527637822603658699938581184513".to_string(),
                },
            );
            deps
        },
        dyns: {
            let mut dyns = HashMap::<String, Dynamic>::new();
            dyns.insert(
                "gen.cl".to_string(),
                Dynamic {
                    dockerfile: "gen.Dockerfile".to_string(),
                    args: "".to_string(),
                },
            );
            dyns
        },
        main: "main.cl".to_string(),
    }
}

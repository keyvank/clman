use crate::*;
use std::fs;
use std::path::Path;
use yaml_rust::{yaml, EmitError, ScanError, Yaml, YamlEmitter, YamlLoader};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Emit Error: {0}")]
    Emit(#[from] EmitError),
    #[error("Scan Error: {0}")]
    Scan(#[from] ScanError),
}
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone)]
pub struct Config {
    pub version: String,
    pub deps: Vec<String>,
    pub main: String,
}

impl From<Config> for Yaml {
    fn from(conf: Config) -> Self {
        let mut y = yaml::Hash::new();
        y.insert(
            Yaml::String("version".to_string()),
            Yaml::String(conf.version),
        );
        y.insert(
            Yaml::String("deps".to_string()),
            Yaml::Array(
                conf.deps
                    .iter()
                    .map(|d| Yaml::String(d.to_string()))
                    .collect(),
            ),
        );
        y.insert(Yaml::String("main".to_string()), Yaml::String(conf.main));
        Yaml::Hash(y)
    }
}

pub fn write_config(root: &Path, conf: Config) -> ConfigResult<()> {
    let mut conf_str = String::new();
    let mut emitter = YamlEmitter::new(&mut conf_str);
    emitter.dump(&Yaml::from(conf.clone()))?;
    fs::write(root.join("clman.yaml"), conf_str)?;
    Ok(())
}

pub fn read_config() -> ConfigResult<Config> {
    let config = &YamlLoader::load_from_str(&fs::read_to_string("clman.yaml")?[..])?[0];
    Ok(Config {
        version: config["version"].as_str().unwrap().to_string(),
        deps: config["deps"]
            .as_vec()
            .unwrap()
            .iter()
            .map(|y| y.as_str().unwrap().to_string())
            .collect(),
        main: config["main"].as_str().unwrap().to_string(),
    })
}

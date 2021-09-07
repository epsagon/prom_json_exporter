use serde::Deserialize;
use std::fs;
use std::io::Error;

#[derive(Deserialize)]
pub struct MetricSelector {
    pub name: String,
    pub selector: String
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub metrics: Vec<MetricSelector>
}

#[derive(Debug)]
pub enum ConfigError {
    IOError(std::io::Error),
    YamlError(serde_yaml::Error)
}

impl ConfigFile {
    pub fn from_file(path: &str) -> Result<ConfigFile, ConfigError> {
        let contents = fs::read_to_string(path)
                        .or_else(|err| Err(ConfigError::IOError(err)))?;
        let config :ConfigFile = serde_yaml::from_str(&contents)
            .or_else(|err| Err(ConfigError::YamlError(err)))?;

        Ok(config)
    }

    pub fn validate_config_file(path: &str) -> Result<(), ConfigError> {
        if let Err(err) = ConfigFile::from_file(path) {
            Err(err)
        }
        else {
            Ok(())
        }
    }
}
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct GlobalLabel {
    pub name: String,
    pub selector: String
}

#[derive(Deserialize)]
pub struct ConfigFile {
    pub gauge_field: String,
    pub global_labels: Option<Vec<GlobalLabel>>
}

#[derive(Debug)]
pub enum ConfigError {
    IOError(std::io::Error),
    YamlError(serde_yaml::Error)
}

impl ConfigFile {
    pub fn from_str(yml_str: &str) -> Result<ConfigFile, ConfigError> {
        let config :ConfigFile = serde_yaml::from_str(&yml_str)
            .or_else(|err| Err(ConfigError::YamlError(err)))?;

        Ok(config)
    }

    pub fn from_file(path: &str) -> Result<ConfigFile, ConfigError> {
        let contents = fs::read_to_string(path)
                        .or_else(|err| Err(ConfigError::IOError(err)))?;
        ConfigFile::from_str(&contents)
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
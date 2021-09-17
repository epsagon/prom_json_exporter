use std::error::Error;
use std::fmt;
use std::process::Stdio;
use std::process::Output;
use std::process::Command;
use std::io::prelude::*;

use crate::utils;

#[derive(Clone)]
pub struct Jq {}

impl Jq {
    pub fn new() -> Result<Self, std::io::Error> {
        let jq = Self {};

        if let Err(err) = jq.version() {
            Err(err)
        }
        else {
            Ok(jq)
        }
    }

    pub fn version(&self) -> Result<Output, std::io::Error> {
        Command::new("jq")
                .arg("--version")
                .output()
    }

    pub fn resolve_raw(&self, json_payload: &str, jq_query: &str) -> Result<String, std::io::Error> {
        let jq_process = Command::new("jq")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg(jq_query)
            .spawn()?;

        jq_process.stdin.unwrap().write_all(json_payload.as_bytes())?;

        let mut filtered_json = String::new();
        jq_process.stdout.unwrap().read_to_string(&mut filtered_json)?;

        Ok(filtered_json)
    }

    pub fn resolve_json_scalar_value(&self, json_payload: &str, jq_query: &str) -> Result<String, JqError> {
        let raw_json_value = self.resolve_raw(json_payload, jq_query)?;
        let json_value : serde_json::Value = serde_json::from_str(&raw_json_value)?;
        match utils::json_value_to_str(&json_value) {
            Some(val) => Ok(val),
            None => {
                let err_msg = format!("Expected scalar value. Found {}", json_value.to_string()).to_string();
                let err = ValueError { message: err_msg };
                Err(JqError::NonScalarValueError(err))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueError {
    pub message: String
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ValueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

#[derive(Debug)]
pub enum JqError {
    IoError(std::io::Error),
    ParserError(serde_json::Error),
    NonScalarValueError(ValueError)
}

impl From<std::io::Error> for JqError {
    fn from(io_err: std::io::Error) -> Self {
        JqError::IoError(io_err)
    }
}

impl From<serde_json::Error> for JqError {
   fn from(serde_error: serde_json::Error) -> Self {
       JqError::ParserError(serde_error)
   }
}

impl From<ValueError> for JqError {
    fn from(value_err: ValueError) -> Self {
        JqError::NonScalarValueError(value_err)
    }
}

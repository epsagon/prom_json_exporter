use std::process::Stdio;
use std::process::Output;
use std::process::Command;
use std::io::prelude::*;

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

    pub fn resolve(&self, json_payload: &str, query: &str) -> Result<String, std::io::Error> {
        let jq_process = Command::new("jq")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg(query)
            .spawn()?;

        jq_process.stdin.unwrap().write_all(json_payload.as_bytes())?;

        let mut filtered_json = String::new();
        jq_process.stdout.unwrap().read_to_string(&mut filtered_json)?;

        Ok(filtered_json)
    }
}
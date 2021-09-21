use serde_json::{Map, Value};

use crate::{config_file::Include, prom_label::PromLabel};

pub fn custom_include_label(custom_label_json_object: &Map<String, Value>, selector: &str, include_config: &Include) -> PromLabel {
    let label_name = custom_label_json_object.keys().find(|key| selector.contains(key.as_str())).unwrap();
    PromLabel::new(include_config.label_name.to_string(), label_name.to_string())
}
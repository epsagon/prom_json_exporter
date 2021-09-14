use serde_json::Map;
use serde_json::{Value, Result};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use crate::config_file::ConfigFile;
use crate::prom_label::PromLabel;
use crate::prom_metric::PromMetric;

pub struct Payload {
    json_payload: String,
    config: ConfigFile
}

impl Payload {
    pub fn new(json: String, config: ConfigFile) -> Self {
        Self {
            json_payload: json,
            config: config
        }
    }

    fn json_value_to_str(&self, value: &Value) -> Option<String> {
        if value.is_string() {
            let value_str = value.as_str().unwrap().to_lowercase();
            if value_str == "ok" {
                return Some("1".into())
            }
            else if value == "error" {
                return Some("0".into())
            }
            else {
                return None;
            }
        }
        else if value.is_f64() {
            return Some(value.as_f64().unwrap().to_string())
        }
        else if value.is_i64() {
            return Some(value.as_i64().unwrap().to_string())
        }
        return None
    }

    pub fn json_to_metrics(&self) -> Result<Vec<PromMetric>> {
        let json_object: HashMap<String, Value> = serde_json::from_str(&self.json_payload)?;

        let mut metrics = vec![];

        for root_key in json_object {
            let root_key_name = root_key.0.to_case(Case::Snake);

            let child_object = root_key.1.as_object().unwrap();

            for child_key in child_object {
                if child_key.0.eq(&self.config.gauge_field) {
                    if let Some(prom_value) = self.json_value_to_str(&child_key.1) {
                        let metric_name = format!("{}_{}", root_key_name, child_key.0.to_case(Case::Snake));
                        metrics.push(PromMetric::new(metric_name, Some(prom_value), None));
                    }
                }
            }
        }

        Ok(metrics)
    }

}

#[cfg(test)]
mod tests {
    use crate::{config_file::{self, ConfigFile}, payload::Payload};

    fn json_with_single_property() -> String {
        r#"{
            "network": {
                "status": "OK"
            }
        }"#.to_string()
    }

    fn config() -> ConfigFile {
        let yaml_str = r#"gauge_field: status"#;
        config_file::ConfigFile::from_str(yaml_str).unwrap()
    }

    #[test]
    fn convert_json_object_with_correct_status_field_config() {
        let json_str = json_with_single_property();
        let payload = Payload::new(json_str, config());
        let metrics = payload.json_to_metrics().unwrap();
        assert_eq!(metrics[0].name, "network_status");
        assert_eq!(metrics[0].value, Some("1".into()));
    }
}

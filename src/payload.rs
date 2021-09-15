use serde_json::{Value, Result};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use crate::config_file::ConfigFile;
use crate::jq::Jq;
use crate::prom_label::PromLabel;
use crate::prom_metric::PromMetric;
use crate::utils;

pub struct Payload {
    full_json_document: String,
    payload_document: String,
    config: ConfigFile,
    jq: Jq
}

impl Payload {
    pub fn new(json: String, json_entry_point: Option<String>, config: ConfigFile) -> Self {
        let jq = Jq::new().unwrap();
        let default_query = ".".to_string(); // `.` is the jq filter that returns the entire document
        let payload_document = jq.resolve(&json, &json_entry_point.unwrap_or(default_query)).unwrap(); //TODO: "HANDLE THIS ERROR ACCORDINGLY"

        Self {
            jq: jq,
            full_json_document: json,
            config: config,
            payload_document: payload_document
        }
    }

    fn fetch_global_metric_labels(&self) -> Vec<PromLabel> {
        let mut labels = vec!();
        for global_label in self.config.global_labels.as_ref().unwrap() {
            let raw_value = self.jq.resolve(&self.full_json_document, &global_label.selector).unwrap();
            labels.push(PromLabel::new(global_label.name.to_string(), raw_value.trim().to_string()));
        }
        labels
    }

    pub fn json_to_metrics(&self) -> Result<Vec<PromMetric>> {
        let json_object: HashMap<String, Value> = serde_json::from_str(&self.payload_document)?;

        let mut metrics = vec![];

        let global_labels = if self.config.global_labels.is_some() {
            Some(self.fetch_global_metric_labels())
        } else {
            None
        };

        for root_key in json_object {
            let root_key_name = root_key.0.to_case(Case::Snake);
            let child_object = root_key.1.as_object().unwrap();

            let mut new_metric = None;

            let gauge_field = self.config.gauge_field.to_string();

            let mut labels = vec!();

            for child_key in child_object.iter().filter(|kv| kv.0.ne(&gauge_field)) {
                if child_key.1.is_object() {
                    //We skip objects by default to avoid deep nesting
                    continue;
                }
                if child_key.1.is_number() || child_key.1.is_string() || child_key.1.is_boolean() {
                    let label_name = child_key.0.to_case(Case::Snake);
                    if let Some(prom_value) = utils::json_value_to_str(child_key.1) {
                        labels.push(PromLabel::new(label_name, prom_value));
                    }
                }
            }

            if let Some(gauge_field) = child_object.iter().find(|(name, _value)| name.to_string().eq(&gauge_field)) {
                if let Some(prom_value) = utils::json_value_to_str(gauge_field.1) {
                    let metric_name = format!("{}_{}", root_key_name, gauge_field.0.to_case(Case::Snake));

                    let metric_labels = if labels.len() > 0 && global_labels.is_some() {
                        let mut l = global_labels.clone().unwrap();
                        l.append(&mut labels);
                        Some(l)
                    } else {
                        global_labels.clone()
                    };

                    new_metric = Some(PromMetric::new(metric_name, Some(prom_value), metric_labels));
                }
            }

            if let Some(m) = new_metric {
                metrics.push(m);
            }
        }

        Ok(metrics)
    }

}

#[cfg(test)]
mod tests {
    use crate::{config_file::{self, ConfigFile}, payload::Payload, prom_metric::PromMetric};

    fn full_json_file() -> String {
        r#"{
            "environment": "production",
            "id": "xyz",
            "components": {
                "network": {
                    "status": "OK",
                    "status_upstream": "active",
                    "has_ip_addresses": true,
                    "use_ip_v6": false,
                    "upstream_endpoints": 54
                }
            }
        }"#.to_string()
    }

    fn config() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
global_labels:
    - name: environment
      selector: .environment
    - name: id
      selector: .id
"#;
        config_file::ConfigFile::from_str(yaml_str).unwrap()
    }

    fn create_metrics() -> Vec<PromMetric> {
        let json_str = full_json_file();
        let payload = Payload::new(json_str, Some(".components".into()), config());
        payload.json_to_metrics().unwrap()
    }

    /*
    TODO
    - Test what happens when no `json_entry_point` gets supplied
    */

    #[test]
    fn convert_json_object_with_correct_status_field_config() {
        let metrics = create_metrics();
        assert_eq!(metrics[0].name, "network_status");
        assert_eq!(metrics[0].value, Some("1".into()));
    }

    #[test]
    fn convert_full_json_file_extract_global_labels() {
        let metrics = create_metrics();
        assert_eq!(metrics[0].name, "network_status");
        assert_eq!(metrics[0].value, Some("1".into()));

        let labels = metrics[0].labels.as_ref().unwrap();

        let l1 = labels.into_iter().find(|l| l.name == "environment").unwrap();
        assert_eq!(l1.value, "\"production\"");

        let l2 = labels.into_iter().find(|l| l.name == "id").unwrap();
        assert_eq!(l2.value, "\"xyz\"");
    }

    #[test]
    fn convert_full_json_file_extract_additional_attribute_names() {
        let metrics = create_metrics();
        let mut label_names = metrics[0].labels
                .as_ref()
                .unwrap()
                .iter()
                .map(|label| label.name.to_string())
                .collect::<Vec<String>>();

        label_names.sort_by(|a, b| a.cmp(b));

        assert_eq!(label_names, vec![
            "environment",
            "has_ip_addresses",
            "id",
            "status_upstream",
            "upstream_endpoints",
            "use_ip_v6"
        ]);
    }

    #[test]
    fn convert_full_json_file_extract_additional_attribute_values() {
        let metrics = create_metrics();
        let mut labels = metrics[0].labels
                .as_ref()
                .unwrap()
                .iter()
                .collect::<Vec<_>>();

        labels.sort_by(|a, b| a.name.cmp(&b.name));

        let values = labels.iter()
                                    .map(|label| label.value.to_string())
                                    .collect::<Vec<_>>();
        assert_eq!(values, vec![
            "\"production\"",
            "true",
            "\"xyz\"",
            "\"active\"",
            "54",
            "false"
        ]);
    }
}

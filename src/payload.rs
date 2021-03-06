use serde_json::Value;
use convert_case::{Case, Casing};
use std::collections::HashMap;
use crate::config_file::ConfigFile;
use crate::custom_include::processor::CustomIncludeProcessor;
use crate::jq::Jq;
use crate::json_object_processor::JsonObjectProcessor;
use crate::prom_label::PromLabel;
use crate::prom_metric::PromMetric;
use crate::utils;
use crate::selector_error::SelectorError;
use crate::payload_error::PayloadError;

pub struct Payload<'a> {
    full_json_document: String,
    payload_document: String,
    config: &'a ConfigFile,
    jq: Jq
}

impl<'a> Payload<'a> {
    pub fn new(json: String, json_entry_point: Option<String>, config: &'a ConfigFile) -> Self {
        let jq = Jq::new().unwrap();
        let default_query = ".".to_string(); // `.` is the jq filter that returns the entire document
        let payload_document = jq.resolve_raw(&json, &json_entry_point.unwrap_or(default_query)).unwrap(); //TODO: "HANDLE THIS ERROR ACCORDINGLY"

        Self {
            jq: jq,
            full_json_document: json,
            config: config,
            payload_document: payload_document
        }
    }

    pub fn json_to_metrics(&self) -> Result<Vec<PromMetric>, PayloadError> {
        let json_object: HashMap<String, Value> = serde_json::from_str(&self.payload_document)?;
        let mut metrics = vec![];

        let global_labels = if self.config.global_labels.is_some() {
            Some(self.fetch_global_metric_labels()?)
        } else {
            None
        };

        for root_key in json_object {
            if root_key.1.is_object() {
                let processor = JsonObjectProcessor::new(root_key.0, root_key.1, global_labels.clone()).unwrap();
                if let Some(mut m) = processor.visit(&self.config) {
                    metrics.append(&mut m);
                }
            }
            else if root_key.1.is_number() {
                if let Some(m) = self.visit_number(root_key, &global_labels) {
                    metrics.push(m);
                }
            }
        }

        if self.config.includes.is_some() {
            let include_processor = CustomIncludeProcessor::new(
                self.payload_document.to_string(),
                self.config.clone(),
                    global_labels.clone(),
                    self.jq.clone()
            );
            metrics.append(&mut include_processor.process()?);
        }

        Ok(metrics)
    }

    fn fetch_global_metric_labels(&self) -> Result<Vec<PromLabel>, SelectorError> {
        let mut labels = vec!();
        for global_label in self.config.global_labels.as_ref().unwrap() {
            let raw_value = self.jq.resolve_json_scalar_value(
                &self.full_json_document,
                &global_label.selector
            );

            match raw_value {
                Ok(val) => labels.push(PromLabel::new(global_label.name.to_string(), val.trim().to_string())),
                Err(err) => return Err(SelectorError::new("Failed to fetch global metric", Some(err)))
            }
        }
        Ok(labels)
    }

    fn visit_number(&self, json_value: (String, Value), global_labels: &Option<Vec<PromLabel>>) -> Option<PromMetric> {
        let metric_name = json_value.0.to_case(Case::Snake);
        if let Some(num) = utils::json_number_to_i64(&json_value.1) {
            Some(PromMetric::new(metric_name, Some(num), global_labels.clone()))
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use std::error::Error;
    use crate::{config_file::{self, ConfigFile}, payload::Payload, prom_metric::PromMetric};
    use super::PayloadError;
    use assert_matches::assert_matches;

    fn full_json_file() -> String {
        r#"{
            "environment": "production",
            "id": "xyz",
            "last_refresh_epoch": 1631046901,
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

    fn json_with_several_components() -> String {
        r#"{
            "environment": "production",
            "id": "xyz",
            "last_refresh_epoch": 1631046901,
            "components": {
                "network": {
                    "status": "OK",
                    "status_upstream": "active",
                    "has_ip_addresses": true,
                    "use_ip_v6": false,
                    "upstream_endpoints": 54
                },
                "router": {
                    "status": "Warning",
                    "num_active_uplinks": 1,
                    "num_uplinks": 2,
                    "backend": {
                        "back1": {
                            "status": "warning",
                            "total_count": 2,
                            "healthy_count": 1,
                            "unhealthy_count": 1
                        },
                        "back2": {
                            "status": "warning",
                            "total_count": 2,
                            "healthy_count": 1,
                            "unhealthy_count": 1
                        }
                    }
                }
            }
        }"#.to_string()
    }

    fn json_with_numerical_status() -> String {
        r#"{
            "environment": "production",
            "id": "xyz",
            "last_refresh_epoch": 1631046901,
            "components": {
                "network": {
                    "status": 1,
                    "status_upstream": "active",
                    "has_ip_addresses": true,
                    "use_ip_v6": false,
                    "upstream_endpoints": 54
                },
                "router": {
                    "status": 2,
                    "num_active_uplinks": 1,
                    "num_uplinks": 2,
                    "backend": {
                        "back1": {
                            "status": 2,
                            "total_count": 2,
                            "healthy_count": 1,
                            "unhealthy_count": 1
                        },
                        "back2": {
                            "status": 2,
                            "total_count": 2,
                            "healthy_count": 1,
                            "unhealthy_count": 1
                        }
                    }
                }
            }
        }"#.to_string()
    }

    fn json_with_numeric_values() -> String {
        r#"{
            "environment": "production",
            "id": "xyz",
            "last_refresh_epoch": 1631046901,
            "api_http_requests_total": 456,
            "http_requests": 2,
            "components": {
                "http_server": {
                    "up": true
                }
            }
        }"#.to_string()
    }

    fn config_without_gauge_mapping() -> ConfigFile {
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

    fn config_with_non_existing_global_labels() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
global_labels:
    - name: Does not exist
      selector: .does_not_exist
    - name: id
      selector: .id
"#;
        config_file::ConfigFile::from_str(yaml_str).unwrap()
    }

    fn config_with_gauge_mapping() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
gauge_field_values:
  - warning
  - ok
global_labels:
    - name: environment
      selector: .environment
    - name: id
      selector: .id
"#;
        config_file::ConfigFile::from_str(yaml_str).unwrap()
    }

    fn config_with_custom_include_and_no_gauge_field_values() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
global_labels:
    - name: environment
      selector: .environment
    - name: id
      selector: .id
includes:
    - name: router_backend_status
      label_name: backend
      label_selector: .router.backend
      selector:
        - ".router.backend.back1"
        - ".router.backend.back2"
"#;
        config_file::ConfigFile::from_str(yaml_str).unwrap()
    }

    fn config_with_custom_includes() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
gauge_field_values:
  - warning
  - ok
global_labels:
    - name: environment
      selector: .environment
    - name: id
      selector: .id
includes:
    - name: router_backend_status
      label_name: backend
      label_selector: .router.backend
      selector:
        - ".router.backend.back1"
        - ".router.backend.back2"
"#;
        config_file::ConfigFile::from_str(yaml_str).unwrap()
    }

    fn config_with_custom_includes_and_invalid_label_selector() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
gauge_field_values:
  - warning
  - ok
global_labels:
    - name: environment
      selector: .environment
    - name: id
      selector: .id
includes:
    - name: router_backend_status
      label_name: backend
      label_selector: .does_not_exist
      selector:
        - ".router.backend.back1"
        - ".router.backend.back2"
"#;
        config_file::ConfigFile::from_str(yaml_str).unwrap()
    }

    fn create_metrics() -> Vec<PromMetric> {
        let json_str = full_json_file();
        let config = config_without_gauge_mapping();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        payload.json_to_metrics().unwrap()
    }

    #[test]
    fn convert_json_object_no_entry_point() {
        let json_str = json_with_numeric_values();
        let config = config_without_gauge_mapping();
        let payload = Payload::new(json_str, None, &config);
        let mut payload_names= payload.json_to_metrics()
                                        .unwrap()
                                        .iter()
                                        .map(|metric| metric.name.to_string())
                                        .collect::<Vec<_>>();

        payload_names.sort_by(|a, b| a.cmp(&b));

        assert_eq!(payload_names, vec![
            "api_http_requests_total",
            "http_requests",
            "last_refresh_epoch"
        ]);
    }

    #[test]
    fn convert_json_object_invalid_global_label_selector() {
        //We want to test what happens when we try to fetch global labels from the json
        //that do not exist
        let json_str = json_with_numeric_values();
        let config = config_with_non_existing_global_labels();
        let payload = Payload::new(json_str, None, &config);
        match payload.json_to_metrics().unwrap_err() {
            PayloadError::SelectorError(err) => {
                assert!(err.source().is_some());
            },
            _ => {
                assert!(false);
            }
        }
    }

    #[test]
    fn convert_json_object_no_entry_point_does_not_convert_child_object() {
        let json_str = json_with_numeric_values();
        let config = config_without_gauge_mapping();
        let payload = Payload::new(json_str, None, &config);
        let metrics = payload.json_to_metrics().unwrap();
        let component_metric = metrics.iter().find(|m| m.name == "components");

        assert!(component_metric.is_none());
    }

    #[test]
    fn convert_json_object_with_correct_status_field_config() {
        let metrics = create_metrics();
        assert_eq!(metrics[0].name, "network_status");
        assert_eq!(metrics[0].value, Some(1));
    }

    #[test]
    fn convert_full_json_file_extract_global_labels() {
        let metrics = create_metrics();
        assert_eq!(metrics[0].name, "network_status");
        assert_eq!(metrics[0].value, Some(1));

        let labels = metrics[0].labels.as_ref().unwrap();

        let l1 = labels.into_iter().find(|l| l.name == "environment").unwrap();
        assert_eq!(l1.value, "production");

        let l2 = labels.into_iter().find(|l| l.name == "id").unwrap();
        assert_eq!(l2.value, "xyz");
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
            "production",
            "true",
            "xyz",
            "active",
            "54",
            "false"
        ]);
    }

    #[test]
    fn convert_full_json_with_root_entry_point_only_converts_numeric() {
        let json_str = full_json_file();
        let config = config_without_gauge_mapping();
        let payload = Payload::new(json_str, Some(".".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();
        assert_eq!(metrics[0].name, "last_refresh_epoch");
        assert_eq!(metrics[0].value, Some(1631046901));
    }

    #[test]
    fn convert_full_json_with_root_entry_point_has_global_attributes() {
        let json_str = full_json_file();
        let config = config_without_gauge_mapping();
        let payload = Payload::new(json_str, Some(".".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();
        let labels = metrics[0].labels.as_ref().unwrap();

        assert_eq!(labels[0].name, "environment");
        assert_eq!(labels[1].name, "id");
    }

    #[test]
    fn convert_json_ensure_one_metric_per_gauge_value() {
        let json_str = json_with_several_components();
        let config = config_with_gauge_mapping();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();
        assert_eq!(metrics.len(), 4);
    }

    #[test]
    fn convert_json_ensure_metric_per_gauge_value_has_correct_label() {
        let json_str = json_with_several_components();
        let config = config_with_gauge_mapping();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();

        let first = metrics[0].labels.as_ref().unwrap();
        let second = metrics[1].labels.as_ref().unwrap();

        let first_status = first.iter().find(|label| label.name == "status");
        assert_eq!(first_status.unwrap().value, "warning");
        let second_status = second.iter().find(|label| label.name == "status");
        assert_eq!(second_status.unwrap().value, "ok");
    }

    #[test]
    fn convert_json_ensure_metric_per_gauge_value_has_correct_flag() {
        let json_str = json_with_several_components();
        let config = config_with_gauge_mapping();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();

        assert_eq!(metrics.len(), 4);

        let router_metrics = metrics.iter().filter(|m| m.name == "router_status").collect::<Vec<_>>();
        assert_eq!(router_metrics.len(), 2);

        let has_one_flag = router_metrics.iter().any(|m| m.value == Some(1));
        let has_zero_flag = router_metrics.iter().any(|m| m.value == Some(0));

        assert!(has_one_flag);
        assert!(has_zero_flag);
    }


    #[test]
    fn convert_json_ensure_custom_includes_metric_has_correct_name() {
        let json_str = json_with_several_components();
        let config = config_with_custom_includes();
        let payload = Payload::new(json_str, Some(".components".into()),&config);
        let metrics = payload.json_to_metrics().unwrap();

        let custom_includes = metrics.iter().filter(|metric| metric.name == "router_backend_status")
        .collect::<Vec<_>>();

        //1 for each selector (2 total) * 2 gauge_field_values
        assert_eq!(custom_includes.len(), 4);
    }

    #[test]
    fn convert_json_ensure_custom_includes_metric_has_gauge_label() {
        let json_str = json_with_several_components();
        let config = config_with_custom_includes();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();
        let custom_includes = metrics.iter().filter(|metric| metric.name == "router_backend_status")
                                .collect::<Vec<_>>();
        let has_custom_label = custom_includes.iter().all(|m| {
            m.labels.as_ref().unwrap().iter().any(|l| l.name == "status")
        });

        assert!(has_custom_label);
    }

    #[test]
    fn convert_json_ensure_custom_includes_has_include_label() {
        let json_str = json_with_several_components();
        let config = &config_with_custom_includes();
        let payload = Payload::new(json_str, Some(".components".into()), config);
        let metrics = payload.json_to_metrics().unwrap();
        let custom_includes = metrics.iter().filter(|metric| metric.name == "router_backend_status")
                                .collect::<Vec<_>>();
        let has_include_label = custom_includes.iter().all(|m|{
            m.labels.as_ref().unwrap()
            .iter()
            .any(|l| l.name == "backend" && l.value.starts_with("back"))
        });

        assert!(has_include_label);
    }

    #[test]
    fn convert_json_custom_include_with_invalid_selector_returns_error() {
        let json_str = json_with_several_components();
        let config = config_with_custom_includes_and_invalid_label_selector();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        let metrics_or_error= payload.json_to_metrics();
        assert!(metrics_or_error.is_err());
        assert_matches!(metrics_or_error.unwrap_err(), PayloadError::SelectorError(_));
    }

    #[test]
    fn convert_json_custom_include_without_gauge_values_returns_four_metrics() {
        let json_str =  json_with_numerical_status();
        let config = config_with_custom_include_and_no_gauge_field_values();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();
        //We need 2 metrics for everything directly under `components`
        //Plus two for the custom include
        assert_eq!(metrics.len(), 4);
    }

    #[test]
    fn convert_json_custom_include_without_gauge_values_custom_include_metrics_have_the_right_format() {
        let json_str =  json_with_numerical_status();
        let config = config_with_custom_include_and_no_gauge_field_values();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();
        let metrics = metrics.iter()
                .filter(|m| m.name == "router_backend_status")
                .collect::<Vec<_>>();
        assert_eq!(metrics.len(), 2);

        assert!(metrics[0].labels.as_ref().unwrap().iter().find(|l| l.name == "environment").is_some());
        assert!(metrics[0].labels.as_ref().unwrap().iter().find(|l| l.name == "id").is_some());
        assert!(metrics[0].labels.as_ref().unwrap().iter().find(|l| l.name == "backend").is_some());
        assert!(metrics[0].labels.as_ref().unwrap().iter().find(|l| l.name == "status").is_none());

        assert!(metrics[1].labels.as_ref().unwrap().iter().find(|l| l.name == "environment").is_some());
        assert!(metrics[1].labels.as_ref().unwrap().iter().find(|l| l.name == "id").is_some());
        assert!(metrics[0].labels.as_ref().unwrap().iter().find(|l| l.name == "backend").is_some());
        assert!(metrics[1].labels.as_ref().unwrap().iter().find(|l| l.name == "status").is_none());
    }

    #[test]
    fn convert_json_with_custom_include_no_duplicate_status_tags() {
        let json_str = json_with_several_components();
        let config = config_with_custom_includes();
        let payload = Payload::new(json_str, Some(".components".into()), &config);
        let metrics = payload.json_to_metrics().unwrap();

        for metric in metrics {
            assert_eq!(metric.labels.as_ref().unwrap().iter().filter(|l| l.name == "status").count(), 1, "metric {} has more than one status label", metric.name);
        }
    }
}

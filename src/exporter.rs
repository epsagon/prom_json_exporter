use convert_case::{Case, Casing};

use crate::{config_file::ConfigFile, prom_metric::PromMetric};

pub struct Exporter<'a> {
    config: &'a ConfigFile,
    metrics: Vec<PromMetric>
}

impl<'a> Exporter<'a> {
    pub(crate) fn new(config: &'a ConfigFile, metrics: Vec<PromMetric>) -> Self {
        Self {
            config: config,
            metrics: metrics
        }
    }

    pub(crate) fn generate_metrics(&self) -> String {
        let mut all_converted_metrics = vec!();

        for metric in &self.metrics {
            all_converted_metrics.push(self.metric_to_string(metric));
        }

        all_converted_metrics.join("\n").to_string()
    }

    fn metric_name(&self, metric: &PromMetric) -> String {
        if let Some(metric_prefix) = &self.config.global_prefix {
            format!("{}_{}", metric_prefix.to_case(Case::Snake), metric.name.to_string())
        }
        else {
            metric.name.to_string()
        }
    }

    fn metric_to_string(&self, metric: &PromMetric) -> std::string::String {
        if metric.labels.is_none() {
            format!("{} {}", self.metric_name(metric), metric.value.as_ref().unwrap_or(&0))
        }
        else {
            let labels = metric.labels
                .as_ref()
                .unwrap()
                .iter()
                .map(|label| label.to_string())
                .collect::<Vec<_>>()
                .join(",");

            format!("{}{{{}}} {}", self.metric_name(metric), labels, metric.value.as_ref().unwrap_or(&0))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{config_file::{self, ConfigFile}, payload::Payload};

    use super::Exporter;

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
        }"#
        .to_string()
    }

    fn config_without_global_prefix() -> ConfigFile {
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

    fn config_with_global_prefix() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
gauge_field_values:
  - warning
  - ok
global_prefix: prom_test
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

    fn config_with_dashed_prefix() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
gauge_field_values:
  - warning
  - ok
global_prefix: prom-test
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

    fn config_with_camel_case_prefix() -> ConfigFile {
        let yaml_str = r#"
gauge_field: status
gauge_field_values:
  - warning
  - ok
global_prefix: promTest
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

    fn generate_metrics(config: &ConfigFile) -> String {
        let json_str = json_with_several_components();
        let payload = Payload::new(
            json_str,
            Some(".components".into()),
            &config,
        );
        let metrics = payload.json_to_metrics().unwrap();
        let exporter = Exporter::new(&config, metrics);
        let metrics_payload = exporter.generate_metrics();
        metrics_payload
    }

    #[test]
    fn export_json_without_global_prefix() {
        let config = config_without_global_prefix();
        let json_str = json_with_several_components();
        let payload = Payload::new(
            json_str,
            Some(".components".into()),
            &config,
        );
        let metrics = payload.json_to_metrics().unwrap();
        let metric_name = metrics[0].name.to_string();
        let exporter = Exporter::new(&config, metrics);
        let metrics_payload = exporter.generate_metrics();
        let lines = metrics_payload.lines().collect::<Vec<_>>();

        assert!(lines[0].starts_with(&metric_name));
    }

    #[test]
    fn export_json_with_global_prefix() {
        let config = config_with_global_prefix();
        let metrics_payload = generate_metrics(&config);

        for metric in metrics_payload.lines() {
            assert!(
                metric.starts_with("prom_test"),
                "Expected metric name {} to start with prefix 'prom_test'",
                metric
            );
        }
    }

    #[test]
    fn export_json_with_dashes_in_prefix_converts_to_snake_case() {
        let json_str = json_with_several_components();
        let config = config_with_dashed_prefix();
        let payload = Payload::new(
            json_str,
            Some(".components".into()),
            &config,
        );
        let metrics = payload.json_to_metrics().unwrap();

        let exporter = Exporter::new(&config, metrics);
        let metrics_payload = exporter.generate_metrics();

        for metric in metrics_payload.lines() {
            assert!(
                metric.starts_with("prom_test"),
                "Expected metric name {} to start with prefix 'prom_test'",
                metric
            );
        }
    }

    #[test]
    fn export_json_global_prefix_camel_case_converts_to_snake_case() {
        let json_str = json_with_several_components();
        let config = config_with_camel_case_prefix();
        let payload = Payload::new(
            json_str,
            Some(".components".into()),
            &config,
        );
        let metrics = payload.json_to_metrics().unwrap();

        let exporter = Exporter::new(&config, metrics);
        let metrics_payload = exporter.generate_metrics();

        for metric in metrics_payload.lines() {
            assert!(
                metric.starts_with("prom_test"),
                "Expected metric name {} to start with prefix 'prom_test'",
                metric
            );
        }
    }
}

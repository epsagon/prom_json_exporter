use convert_case::{Case, Casing};
use serde_json::{Map, Value};
use crate::{config_file::ConfigFile, prom_label::PromLabel, prom_metric::PromMetric, utils};

pub struct JsonObjectProcessor {
    root_key_name: String,
    global_labels: Option<Vec<PromLabel>>,
    child_object: Map<String, Value>
}

impl JsonObjectProcessor {
    pub fn new(root_key_name: String, child_object: Value, global_labels: Option<Vec<PromLabel>>) -> Option<Self> {
        let child_object_map = child_object.as_object()?;

        Some(
            Self {
                root_key_name: root_key_name.to_case(Case::Snake),
                child_object: child_object_map.clone(),
                global_labels: global_labels
            }
        )
    }

    pub fn visit(&self, config: &ConfigFile) -> Option<Vec<PromMetric>> {
        if config.gauge_field_values.is_some() {
            return self.multi_metric_strategy(config)
        }

        if let Some(metric) = self.single_metric_strategy(config) {
            Some(vec!(metric))
        } else {
            None
        }
    }

    fn multi_metric_strategy(&self, config: &ConfigFile) -> Option<Vec<PromMetric>> {
        let gauge_field_name = config.gauge_field.to_string();
        let mut labels = vec!();
        labels.append(&mut self.extract_labels(config, &self.child_object));
        let gauge_field = self.child_object.iter().find(|(name, _value)| name.to_string().eq(&gauge_field_name))?;
        let metric_name = format!("{}_{}", self.root_key_name, gauge_field.0.to_case(Case::Snake));
        let mut metrics = vec!();
        let metric_labels = if labels.len() > 0 && self.global_labels.is_some() {
            let mut l = self.global_labels.clone().unwrap();
            l.append(&mut labels);
            Some(l)
        } else {
            self.global_labels.clone()
        };
        if let Some(gauge_field_values) = &config.gauge_field_values {
            return Some(gauge_field_values.iter()
                .map(|field_value| {
                    let labels = match metric_labels.clone() {
                        Some(mut labels) => {
                            labels.append(&mut vec![PromLabel::new(gauge_field_name.to_string(), field_value.to_string())]);
                            Some(labels)
                        }
                        None => None
                    };

                    let converted_value= utils::json_value_to_str(gauge_field.1).unwrap();
                    if converted_value.to_lowercase() == field_value.to_lowercase() {
                        PromMetric::new(metric_name.to_string(), Some(1), labels)
                    }
                    else {
                        PromMetric::new(metric_name.to_string(), Some(0), labels)
                    }
                })
                .collect::<Vec<_>>())
        }
        else {
            let prom_value = utils::json_value_to_i64(gauge_field.1)?;
            metrics.push(PromMetric::new(metric_name, Some(prom_value), metric_labels));
        }

        Some(metrics)
    }

    fn single_metric_strategy(&self, config: &ConfigFile) -> Option<PromMetric> {
        let gauge_field = config.gauge_field.to_string();
        let mut labels = vec!();
        labels.append(&mut self.extract_labels(config, &self.child_object));
        let gauge_field = self.child_object.iter().find(|(name, _value)| name.to_string().eq(&gauge_field))?;
        let prom_value = utils::json_value_to_i64(gauge_field.1)?;
        let metric_name = format!("{}_{}", self.root_key_name, gauge_field.0.to_case(Case::Snake));
        let metric_labels = if labels.len() > 0 && self.global_labels.is_some() {
            let mut l = self.global_labels.clone().unwrap();
            l.append(&mut labels);
            Some(l)
        } else {
            self.global_labels.clone()
        };
        Some(PromMetric::new(metric_name, Some(prom_value), metric_labels))
    }

    fn extract_labels(&self, config: &ConfigFile, child_object: &serde_json::Map<String, Value>) -> Vec<PromLabel> {
        let gauge_field = config.gauge_field.to_string();
        let mut labels = vec!(); //Vec<PromLabel>;
        for child_key in child_object.iter().filter(|kv| kv.0.ne(&gauge_field)) {
            if child_key.1.is_number() || child_key.1.is_string() || child_key.1.is_boolean() {
                let label_name = child_key.0.to_case(Case::Snake);
                if let Some(prom_value) = utils::json_value_to_str(child_key.1) {
                    labels.push(PromLabel::new(label_name, prom_value));
                }
            }
        }

        labels
    }
}
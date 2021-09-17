use convert_case::{Case, Casing};
use serde_json::{Map, Value};

use crate::{config_file::{self, ConfigFile}, prom_label::PromLabel, prom_metric::PromMetric, utils};

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
        if let Some(metric) = self.single_metric_strategy(config) {
            Some(vec!(metric))
        } else {
            None
        }
    }

    fn multi_metric_strategy(&self, config: &ConfigFile) -> Option<PromMetric> {
        None
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
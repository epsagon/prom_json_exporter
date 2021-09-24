use serde_json::Value;

use crate::{config_file::{ConfigFile, Include}, jq::Jq, prom_label::PromLabel, prom_metric::PromMetric, utils};

use super::error::CustomIncludeError;

pub struct IncludeProcessor<'a> {
    config: &'a ConfigFile,
    include: &'a Include,
    jq: &'a Jq,
    json_document: &'a str,
    global_labels: &'a Option<Vec<PromLabel>>,
}

impl<'a> IncludeProcessor<'a> {
    pub fn new(
        config: &'a ConfigFile,
        include: &'a Include,
        jq: &'a Jq,
        json_document: &'a str,
        global_labels: &'a Option<Vec<PromLabel>>,
    ) -> Self {
        Self {
            config: config,
            include: include,
            jq: jq,
            json_document: json_document,
            global_labels: global_labels,
        }
    }

    pub fn create_metrics(&self) -> Result<Vec<PromMetric>, CustomIncludeError> {
        let mut metrics = vec![];
        for include_selector in &self.include.selector {
            let json_object = self.resolve_json(&include_selector)?;
            metrics.append(&mut self.json_object_to_metric(include_selector, json_object)?);
        }

        Ok(metrics)
    }

    fn json_object_to_metric(&self, include_selector: &str, json_object: Value) -> Result<Vec<PromMetric>, CustomIncludeError> {
        let mut metrics = vec![];
        let mut labels = self.labels(include_selector)?;

        if self.config.has_gauge_values() {
            for gauge_field_value in self.config.gauge_field_values.as_ref().unwrap() {
                labels.push(
                    PromLabel::new(self.config.gauge_field.to_string(), gauge_field_value.to_string())
                );
                let metric_value = self.metric_value(gauge_field_value, &json_object);
                metrics.push(PromMetric::new(
                    self.include.name.to_string(),
                    metric_value,
                    Some(labels.clone())
                ));
            }
        } else {
            if let Some(json_value) = json_object.get(self.config.gauge_field.to_string()) {
                metrics.push(PromMetric::new(
                    self.include.name.to_string(),
                    utils::json_value_to_i64(json_value),
                    Some(labels.clone())
                ));
            }else {
                return Err(CustomIncludeError::SelectorError(format!("Key {} is not present in JSON object", self.config.gauge_field)))
            }
        }

        Ok(metrics)
    }

    fn labels(&self, include_selector: &str) -> Result<Vec<PromLabel>, CustomIncludeError> {
        let label_values = self.fetch_label_values()?;
        let label_value = label_values
            .iter()
            .find(|label_value| include_selector.ends_with(label_value.as_str()))
            .unwrap();

        let label = PromLabel::new(self.include.label_name.to_string(), label_value.to_string());
        let all_labels = if self.global_labels.is_some() {
            let mut l = vec![label];
            l.append(&mut self.global_labels.clone().unwrap());
            l
        } else {
            vec![label]
        };

        Ok(all_labels)
    }

    fn fetch_label_values(&self) -> Result<Vec<String>, CustomIncludeError> {
        let raw_json = self
            .jq
            .resolve_raw(self.json_document, &self.include.label_selector)?;
        let json_value: Value = serde_json::from_str(&raw_json)?;
        if json_value.is_object() {
            let object_keys = json_value.as_object().unwrap().keys();
            Ok(object_keys.map(|k| k.to_string()).collect::<Vec<String>>())
        } else {
            return Err(CustomIncludeError::SelectorError(format!(
                "Selector {} does not point to a valid object",
                self.include.label_selector
            )));
        }
    }

    fn metric_value(&self, gauge_field_value: &str, json_object: &Value) -> Option<i64> {
        if let Some(gauge_value) = json_object.get(&self.config.gauge_field) {
            let value = utils::json_value_to_str(gauge_value).unwrap();
            if value.to_lowercase() == gauge_field_value.to_lowercase() {
                Some(1)
            } else {
                Some(0)
            }
        } else {
            None
        }
    }

    fn resolve_json(&self, selector: &str) -> Result<Value, CustomIncludeError> {
        let json_str = self.jq.resolve_raw(self.json_document, selector)?;
        let json_value = serde_json::from_str(&json_str)?;
        Ok(json_value)
    }
}

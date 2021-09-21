use serde_json::Value;
use crate::{config_file::{ConfigFile, Include}, custom_include::labels, jq::Jq, prom_label::PromLabel, prom_metric::PromMetric, utils};
use super::error::CustomIncludeError;

pub struct CustomIncludeProcessor {
    config: ConfigFile,
    global_labels: Option<Vec<PromLabel>>,
    json_document: String,
    jq_instance: Jq
}

impl CustomIncludeProcessor {
    pub fn new(
                json_doc :String,
                config: ConfigFile,
                global_labels: Option<Vec<PromLabel>>,
                jq: Jq
        ) -> Self {
        Self {
            json_document: json_doc,
            config,
            global_labels: global_labels,
            jq_instance: jq
        }
    }

    pub fn process(&self) -> Result<Vec<PromMetric>, CustomIncludeError> {
        let mut metrics = vec!();

        if let Some(custom_includes) = &self.config.includes {
            for include in custom_includes {
                metrics.append(&mut self.process_include(include)?);
            }
        }

        Ok(metrics)
    }

    fn process_include(&self, include: &Include) -> Result<Vec<PromMetric>, CustomIncludeError>  {
        let mut metrics = vec!();
        let gauge_field = self.config.gauge_field.to_string();

        for selector in &include.selector {
            let json_value = self.resolve_json(selector)?;
            metrics.append(&mut self.process_extracted_json(json_value, &gauge_field, include, selector)?);
        }
        Ok(metrics)
    }

    fn process_extracted_json(&self, json_value: Value, gauge_field: &String, include: &Include, selector: &String) -> Result<Vec<PromMetric>, CustomIncludeError> {
        let mut metrics = vec!();
        if let Some(json_object) = json_value.as_object() {
            if let Some(gauge) = json_object.get(gauge_field) {
                if let Some(config_gauge_field_values) = &self.config.gauge_field_values {
                    let serde_value = self.get_custom_label_json_object(&include.label_selector)?;
                    if let Some(custom_label_json_object) = serde_value.as_object() {//TODO: Handle conversion error {

                        let mut gauge_metrics = config_gauge_field_values
                                .iter()
                                .map(|value|
                                    self.create_metric(
                                            gauge,
                                            gauge_field,
                                            value,
                                            include,
                                            labels::custom_include_label(custom_label_json_object, selector, include)
                                    )
                                )
                                .collect::<Vec<_>>();
                        metrics.append(&mut gauge_metrics);
                    }
                    else {
                        return Err(CustomIncludeError::SelectorError(format!("Invalid Selector {}. Expected Object", &include.label_selector)))
                    }
                }
                else {
                    //TODO: Add tests for this case
                    metrics.push(PromMetric::new(include.name.to_string(),
                        0.into(), self.global_labels.clone()));
                }
            }
            else {
                return Err(CustomIncludeError::SelectorError(format!("{}.{} does not exist", selector, gauge_field)));
            }
        }
        else {
            return Err(CustomIncludeError::SelectorError(format!("{} is not a json object", selector).into()));
        }
        Ok(metrics)
    }

    fn resolve_json(&self, include_selector: &str) -> Result<Value, CustomIncludeError> {
        let json = self.jq_instance.resolve_raw(&self.json_document, include_selector)?;
        let value = serde_json::from_str(&json)?;
        Ok(value)
    }

    fn create_metric(&self, gauge: &Value, gauge_field: &str, config_gauge_field_value: &String, include: &Include, custom_label: PromLabel) -> PromMetric {
        let customized_labels = self.generate_metric_labels(vec![
            custom_label,
            PromLabel::new(gauge_field.to_string(), config_gauge_field_value.to_string())
        ]);
        let object_value = utils::json_value_to_str(gauge).unwrap().to_lowercase();
        let metric_value = if config_gauge_field_value.to_lowercase() == object_value {
            1.into()
        } else {
            0.into()
        };
        PromMetric::new(include.name.to_string(), metric_value, customized_labels)
    }

    fn generate_metric_labels(&self, mut labels: Vec<PromLabel>) -> Option<Vec<PromLabel>> {
        let metric_labels = if labels.len() > 0 && self.global_labels.is_some() {
            let mut l = self.global_labels.clone().unwrap();
            l.append(&mut labels);
            Some(l)
        } else {
            self.global_labels.clone()
        };
        metric_labels
    }

    pub fn get_custom_label_json_object(&self, label_selector: &str) -> Result<Value, CustomIncludeError> {
        let json_str = self.jq_instance.resolve_raw(&self.json_document, label_selector)?;
        match serde_json::from_str(&json_str) {
            Ok(json_object) => Ok(json_object),
            Err(_) => Err(CustomIncludeError::SelectorError(format!("Invalid Custom Include Selector '{}'", label_selector))),
        }
    }
}
use serde_json::Value;

use crate::{config_file::{ConfigFile, Include}, jq::Jq, prom_label::PromLabel, prom_metric::PromMetric, utils};

#[derive(Debug)]
pub enum CustomIncludeError {
    IOError(std::io::Error),
    JsonError(serde_json::Error),
    SelectorError(String)
}

impl From<std::io::Error> for CustomIncludeError {
    fn from(err: std::io::Error) -> Self {
        CustomIncludeError::IOError(err)
    }
}

impl From<serde_json::Error> for CustomIncludeError {
    fn from(err: serde_json::Error) -> Self {
        CustomIncludeError::JsonError(err)
    }
}

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

    fn resolve_json(&self, include_selector: &str) -> Result<Value, CustomIncludeError> {
        let json = self.jq_instance.resolve_raw(&self.json_document, include_selector)?;
        let value = serde_json::from_str(&json)?;
        Ok(value)
    }

    fn process_include(&self, include: &Include) -> Result<Vec<PromMetric>, CustomIncludeError>  {
        let mut metrics = vec!();
        let gauge_field = self.config.gauge_field.to_string();

        for selector in &include.selector {
            let json_value = self.resolve_json(selector)?;
            if let Some(json_object) = json_value.as_object() {
                if let Some(gauge) = json_object.get(&gauge_field) {
                    if let Some(config_gauge_field_values) = &self.config.gauge_field_values {
                        for config_gauge_field_value in config_gauge_field_values {

                            let customized_labels = self.generate_metric_labels(vec![
                                PromLabel::new(gauge_field.to_string(), config_gauge_field_value.to_string())
                            ]);

                            let object_value = utils::json_value_to_str(gauge).unwrap().to_lowercase();
                            let metric_value = if config_gauge_field_value.to_lowercase() == object_value {
                                1.into()
                            } else {
                                0.into()
                            };

                            metrics.push(PromMetric::new(include.name.to_string(),
                                metric_value, customized_labels));
                        }
                    } else {
                        metrics.push(PromMetric::new(include.name.to_string(),
                            0.into(), self.global_labels.clone()));
                    }
                }
                else {
                    return Err(CustomIncludeError::SelectorError(format!("{}.{} does not exist", selector, gauge_field)))
                }
            }
            else {
                return Err(CustomIncludeError::SelectorError(format!("{} is not a json object", selector).into()))
            }
        }
        Ok(metrics)
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
}
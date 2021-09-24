use crate::{jq::Jq, prom_label::PromLabel, prom_metric::PromMetric};
use crate::config_file::{ConfigFile, Include};
use super::{error::CustomIncludeError, include_processor::IncludeProcessor};

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

    fn process_include(&self, include: &Include) -> Result<Vec<PromMetric>, CustomIncludeError> {
        let include_processor = IncludeProcessor::new(
            &self.config,
            include,
            &self.jq_instance,
            &self.json_document,
            &self.global_labels
        );
        let metrics = include_processor.create_metrics()?;
        Ok(metrics)
    }
}
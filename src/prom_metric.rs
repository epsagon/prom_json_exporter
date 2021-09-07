use crate::prom_label::PromLabel;

#[derive(Debug)]
pub struct PromMetric {
    pub name: String,
    pub value: Option<String>,
    pub labels: Option<Vec<PromLabel>>
}

impl PromMetric {
    pub fn new(name: String, value: Option<String>, labels: Option<Vec<PromLabel>>) -> Self {
        Self {
            name: name,
            value: value,
            labels: labels
        }
    }
}

impl ToString for PromMetric {
    fn to_string(&self) -> std::string::String {
        if let Some(value) = &self.value {
            format!("{} {}", self.name, value)
        }
        else {
            let labels = self.labels
                .as_ref()
                .unwrap()
                .iter()
                .map(|label| format!("{}={}", label.name, label.value))
                .collect::<Vec<_>>()
                .join(",");

            format!("{}{{{}}}", self.name, labels)
        }
    }
}
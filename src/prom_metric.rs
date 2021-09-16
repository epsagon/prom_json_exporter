use crate::prom_label::PromLabel;

#[derive(Debug)]
pub struct PromMetric {
    pub name: String,
    pub value: Option<i64>,
    pub labels: Option<Vec<PromLabel>>
}

impl PromMetric {
    pub fn new(name: String, value: Option<i64>, labels: Option<Vec<PromLabel>>) -> Self {
        Self {
            name: name,
            value: value,
            labels: labels
        }
    }
}

impl ToString for PromMetric {
    fn to_string(&self) -> std::string::String {

        if self.labels.is_none() {
            format!("{} {}", self.name, self.value.as_ref().unwrap_or(&0))
        }
        else {
            let labels = self.labels
                .as_ref()
                .unwrap()
                .iter()
                .map(|label| format!("{}={}", label.name, label.value))
                .collect::<Vec<_>>()
                .join(",");

            format!("{}{{{}}} {}", self.name, labels, self.value.as_ref().unwrap_or(&0))
        }
    }
}
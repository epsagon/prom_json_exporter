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

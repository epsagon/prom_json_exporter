use serde_json::Map;
use serde_json::{Value, Result};
use convert_case::{Case, Casing};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PromLabel {
    pub name: String,
    pub value: String
}

impl PromLabel {
    pub fn new (name: String, value: String) -> Self {
        Self {
            name: name,
            value: value
        }
    }
}

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

            format!("{}{{ {} }}", self.name, labels)
        }
    }
}

fn to_labels(child_object: &Map<String, Value>) -> Vec<PromLabel> {
    let mut labels = vec!();

    for (child_key, child_value) in child_object {
        let snake_case_name = child_key.to_case(Case::Snake);
        labels.push(PromLabel::new(snake_case_name, child_value.to_string()));
    }

    labels
}

fn convert_json_array(value: Value, snake_case_name: String) -> Vec<PromMetric> {
    let child_list = value.as_array().unwrap();
    let mut metrics = vec!();

    for label in child_list {
        let mut labels = vec!();
        for (child_key, child_value) in label.as_object().unwrap() {
            labels.push(PromLabel::new(child_key.to_string(), child_value.to_string()));
        }
        metrics.push(PromMetric::new(snake_case_name.to_string(), None, Some(labels)));
    }
    metrics
}

pub fn json_to_metrics(json: String) -> Result<Vec<PromMetric>> {
    let json_object: HashMap<String, Value> = serde_json::from_str(&json)?;

    let mut metrics = vec![];

    for (parent_key, value) in json_object {
        let snake_case_name = parent_key.to_case(Case::Snake);

        if value.is_object() {
            let child_object = value.as_object().unwrap();
            metrics.push(PromMetric::new(snake_case_name.to_string(), None, Some(to_labels(&child_object))));
        }
        else if value.is_array() {
            let mut new_metrics = convert_json_array(value, snake_case_name);
            metrics.append(&mut new_metrics);
        }
        else {
            metrics.push(PromMetric::new(snake_case_name, Some(value.to_string()), None));
        }
    }

    Ok(metrics)
}

#[cfg(test)]
mod tests {

    use crate::payload::json_to_metrics;

    fn nested_json() -> String {
        r#"{"status":"success","code":0,"data":{"UserCount":140,"UserCountActive":23}}"#.to_string()
    }

    fn simple_json() -> String {
        r#"{
        "server_up": "up",
        "backends": 23
      }"#.to_string()
    }

    fn simple_json_with_camel_case() -> String {
        r#"{
        "serverUp": "up",
        "backends": 23
      }"#.to_string()
    }

    fn json_with_list() -> String {
        r#"{
            "counter": 1234,
            "values": [
                {
                    "id": "id-A",
                    "count": 1,
                    "some_boolean": true,
                    "state": "ACTIVE"
                },
                {
                    "id": "id-B",
                    "count": 2,
                    "some_boolean": true,
                    "state": "INACTIVE"
                },
                {
                    "id": "id-C",
                    "count": 3,
                    "some_boolean": false,
                    "state": "ACTIVE"
                }
            ],
            "location": "mars"
        }"#.to_string()
    }


    #[test]
    fn simple_json_converts_numeric() {
        let backend_metric = json_to_metrics(simple_json()).unwrap().into_iter()
                                .find(|x| x.name == "backends")
                                .unwrap();

        assert_eq!(backend_metric.name, "backends");
        assert_eq!(backend_metric.value, Some("23".to_string()));
    }

    #[test]
    fn simple_json_converts_string() {
        let uptime_metric = json_to_metrics(simple_json()).unwrap().into_iter()
                                .find(|x| x.name == "server_up")
                                .unwrap();

        assert_eq!(uptime_metric.name, "server_up");
        assert_eq!(uptime_metric.value, Some("\"up\"".to_string()));
    }

    #[test]
    fn simple_json_converts_camel_case_to_snake_case() {
        let uptime_metric = json_to_metrics(simple_json_with_camel_case()).unwrap().into_iter()
            .find(|x| x.name == "server_up")
            .unwrap();

        assert_eq!(uptime_metric.name, "server_up");
        assert_eq!(uptime_metric.value, Some("\"up\"".to_string()));
    }

    #[test]
    fn complex_json_converts_status() {
        let uptime_metric = json_to_metrics(nested_json()).unwrap().into_iter()
                                .find(|x| x.name == "status")
                                .unwrap();

        assert_eq!(uptime_metric.name, "status");
        assert_eq!(uptime_metric.value, Some("\"success\"".to_string()));
    }

    #[test]
    fn complex_json_converts_code() {
        let uptime_metric = json_to_metrics(nested_json()).unwrap().into_iter()
                                .find(|x| x.name == "code")
                                .unwrap();

        assert_eq!(uptime_metric.name, "code");
        assert_eq!(uptime_metric.value, Some("0".to_string()));
    }

    #[test]
    fn complex_json_convert_data_user_count_label() {
        let metrics = json_to_metrics(nested_json()).unwrap();
        let data_metric = metrics.into_iter()
                                .find(|x| x.name == "data")
                                .unwrap();

        assert_eq!(data_metric.name, "data");
        assert_eq!(data_metric.value, None);

        let user_count_label = data_metric.labels
                                            .unwrap()
                                            .into_iter()
                                            .find(|x| x.name == "user_count")
                                            .unwrap();

        assert_eq!(user_count_label.name, "user_count");
        assert_eq!(user_count_label.value, "140".to_string());
    }

    #[test]
    fn complex_json_convert_user_count_active_label() {
        let metrics = json_to_metrics(nested_json()).unwrap();
        let data_metric = metrics.into_iter()
                                    .find(|x| x.name == "data")
                                    .unwrap();

        let user_count_active_label = data_metric.labels
        .unwrap()
        .into_iter()
        .find(|x| x.name == "user_count_active")
        .unwrap();

        assert_eq!(user_count_active_label.name, "user_count_active");
        assert_eq!(user_count_active_label.value, "23".to_string());
    }

    #[test]
    fn complex_json_convert_list_to_labels_has_three_metrics() {
        let metrics = json_to_metrics(json_with_list()).unwrap();
        let values_metrics = metrics.into_iter()
                                    .filter(|x| x.name == "values");
        assert_eq!(values_metrics.count(), 3);
    }

    #[test]
    fn complex_json_convert_list_to_labels_has_correct_number_of_labels() {
        let metrics = json_to_metrics(json_with_list()).unwrap();
        let values_metrics = metrics.into_iter()
                                    .filter(|x| x.name == "values")
                                    .collect::<Vec<_>>();

        let values_metric = &values_metrics[0];
        assert_eq!(values_metric.labels.as_ref().unwrap().len(), 4);
    }
}

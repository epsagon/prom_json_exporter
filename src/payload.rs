use serde_json::{Value, Result};
use convert_case::{Case, Casing};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PromMetric {
    pub name: String,
    pub value: String,
}

impl PromMetric {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name: name,
            value: value,
        }
    }
}

impl ToString for PromMetric {
    fn to_string(&self) -> std::string::String {
        format!("{} {}", self.name, self.value)
    }
}

pub fn json_to_metrics(json: String) -> Result<Vec<PromMetric>> {
    let json_object: HashMap<String, Value> = serde_json::from_str(&json)?;

    let mut metrics = vec![];

    for (parent_key, value) in json_object {
        let snake_case_name = parent_key.to_case(Case::Snake);
        if !value.is_object() {
            metrics.push(PromMetric::new(snake_case_name, value.to_string()));
        }
        else {
            let child_object = value.as_object().unwrap();
            for (child_key, child_value) in child_object {
                let snake_case_name = format!("{}_{}", parent_key, child_key.to_case(Case::Snake));
                metrics.push(PromMetric::new(snake_case_name, child_value.to_string()));
            }
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


    #[test]
    fn simple_json_converts_numeric() {
        let backend_metric = json_to_metrics(simple_json()).unwrap().into_iter()
                                .find(|x| x.name == "backends")
                                .unwrap();

        assert_eq!(backend_metric.name, "backends");
        assert_eq!(backend_metric.value, "23");


    }

    #[test]
    fn simple_json_converts_string() {
        let uptime_metric = json_to_metrics(simple_json()).unwrap().into_iter()
                                .find(|x| x.name == "server_up")
                                .unwrap();

        assert_eq!(uptime_metric.name, "server_up");
        assert_eq!(uptime_metric.value, "\"up\"");
    }

    #[test]
    fn simple_json_converts_camel_case_to_snake_case() {
        let uptime_metric = json_to_metrics(simple_json_with_camel_case()).unwrap().into_iter()
            .find(|x| x.name == "server_up")
            .unwrap();

        assert_eq!(uptime_metric.name, "server_up");
        assert_eq!(uptime_metric.value, "\"up\"");
    }

    #[test]
    fn complex_json_converts_status() {
        let uptime_metric = json_to_metrics(nested_json()).unwrap().into_iter()
                                .find(|x| x.name == "status")
                                .unwrap();

        assert_eq!(uptime_metric.name, "status");
        assert_eq!(uptime_metric.value, "\"success\"");
    }

    #[test]
    fn complex_json_converts_code() {
        let uptime_metric = json_to_metrics(nested_json()).unwrap().into_iter()
                                .find(|x| x.name == "code")
                                .unwrap();

        assert_eq!(uptime_metric.name, "code");
        assert_eq!(uptime_metric.value, "0");
    }

    #[test]
    fn complex_json_convert_data_user_count() {
        let metrics = json_to_metrics(nested_json()).unwrap();
        let uptime_metric = metrics.into_iter()
                                .find(|x| x.name == "data_user_count")
                                .unwrap();

        assert_eq!(uptime_metric.name, "data_user_count");
        assert_eq!(uptime_metric.value, "140");
    }
}

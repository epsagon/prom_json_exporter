use serde_json::{Value, Result};
use convert_case::{Case, Casing};
use std::collections::HashMap;


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

pub fn json_to_metrics(json: String) -> Result<Vec<PromMetric>> {
    let json_object: HashMap<String, Value> = serde_json::from_str(&json)?;

    let mut metrics = vec![];

    for (key, value) in json_object {
        let snake_case_name = key.to_case(Case::Snake);
        metrics.push(PromMetric::new(snake_case_name, value.to_string()));
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
}

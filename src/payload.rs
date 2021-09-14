use serde_json::Map;
use serde_json::{Value, Result};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use crate::prom_label::PromLabel;
use crate::prom_metric::PromMetric;

pub struct Payload {
    json_payload: String
}

impl Payload {
    pub fn new(json: String) -> Self {
        Self {
            json_payload: json
        }
    }

    fn to_labels(&self, child_object: &Map<String, Value>) -> Vec<PromLabel> {
        let mut labels = vec!();

        for (child_key, child_value) in child_object {
            let snake_case_name = child_key.to_case(Case::Snake);
            labels.push(PromLabel::new(snake_case_name, child_value.to_string()));
        }

        labels
    }

    fn convert_json_array(&self, value: Value, snake_case_name: String) -> Vec<PromMetric> {
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

    pub fn json_to_metrics(&self) -> Result<Vec<PromMetric>> {
        let json_object: HashMap<String, Value> = serde_json::from_str(&self.json_payload)?;

        let mut metrics = vec![];

        for (parent_key, value) in json_object {
            let snake_case_name = parent_key.to_case(Case::Snake);

            if value.is_object() {
                let child_object = value.as_object().unwrap();
                metrics.push(PromMetric::new(snake_case_name.to_string(), None, Some(self.to_labels(&child_object))));
            }
            else if value.is_array() {
                let mut new_metrics = self.convert_json_array(value, snake_case_name);
                metrics.append(&mut new_metrics);
            }
            else {
                metrics.push(PromMetric::new(snake_case_name, Some(value.to_string()), None));
            }
        }

        Ok(metrics)
    }

}

#[cfg(test)]
mod tests {
    use crate::payload::Payload;

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

    //We might have to deal with JSON payloads that contain
    //data that shouldn't be transformed into a metric
    fn json_with_dedicated_metrics_property() -> String {
        r#"{
            "status_msg": "1 out of 2 Edge backends are DOWN. 1 out of 2 Global backends are DOWN.",
            "last_refresh": "2021-09-07 20:35:01 UTC",
            "metrics": {
                "environment": "dev",
                "pop_id": "hiv",
                "status": {
                    "ok": 0,
                    "warning": 1,
                    "critical": 0
                },
                "components": {
                    "network": {
                        "status": {
                            "ok": 1,
                            "warning": 0,
                            "critical": 0
                        }
                    },
                    "exabgp": {
                        "status": {
                            "ok": 1,
                            "warning": 0,
                            "critical": 0
                        }
                    }
                }
            }
        }"#.to_string()
    }

    #[test]
    fn simple_json_converts_numeric() {
        let backend_metric = Payload::new(simple_json()).json_to_metrics().unwrap().into_iter()
                                .find(|x| x.name == "backends")
                                .unwrap();

        assert_eq!(backend_metric.name, "backends");
        assert_eq!(backend_metric.value, Some("23".to_string()));
    }

    #[test]
    fn simple_json_converts_string() {
        let uptime_metric = Payload::new(simple_json()).json_to_metrics().unwrap().into_iter()
                                .find(|x| x.name == "server_up")
                                .unwrap();

        assert_eq!(uptime_metric.name, "server_up");
        assert_eq!(uptime_metric.value, Some("\"up\"".to_string()));
    }

    #[test]
    fn simple_json_converts_camel_case_to_snake_case() {
        let uptime_metric = Payload::new(simple_json_with_camel_case()).json_to_metrics().unwrap().into_iter()
            .find(|x| x.name == "server_up")
            .unwrap();

        assert_eq!(uptime_metric.name, "server_up");
        assert_eq!(uptime_metric.value, Some("\"up\"".to_string()));
    }

    #[test]
    fn complex_json_converts_status() {
        let uptime_metric = Payload::new(nested_json()).json_to_metrics().unwrap().into_iter()
                                .find(|x| x.name == "status")
                                .unwrap();

        assert_eq!(uptime_metric.name, "status");
        assert_eq!(uptime_metric.value, Some("\"success\"".to_string()));
    }

    #[test]
    fn complex_json_converts_code() {
        let uptime_metric = Payload::new(nested_json()).json_to_metrics().unwrap().into_iter()
                                .find(|x| x.name == "code")
                                .unwrap();

        assert_eq!(uptime_metric.name, "code");
        assert_eq!(uptime_metric.value, Some("0".to_string()));
    }

    #[test]
    fn complex_json_convert_data_user_count_label() {
        let metrics = Payload::new(nested_json()).json_to_metrics().unwrap();
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
        let metrics = Payload::new(nested_json()).json_to_metrics().unwrap();
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
        let metrics = Payload::new(json_with_list()).json_to_metrics().unwrap();
        let values_metrics = metrics.into_iter()
                                    .filter(|x| x.name == "values");
        assert_eq!(values_metrics.count(), 3);
    }

    #[test]
    fn complex_json_convert_list_to_labels_has_correct_number_of_labels() {
        let metrics = Payload::new(json_with_list()).json_to_metrics().unwrap();
        let values_metrics = metrics.into_iter()
                                    .filter(|x| x.name == "values")
                                    .collect::<Vec<_>>();

        let values_metric = &values_metrics[0];
        assert_eq!(values_metric.labels.as_ref().unwrap().len(), 4);
    }
}

use serde_json::Value;

pub fn json_value_to_str(value: &Value) -> Option<String> {
    if value.is_string() {
        let value_str = value.as_str().unwrap().to_lowercase();
        if value_str == "ok" {
            return Some("1".into())
        }
        else if value == "error" {
            return Some("0".into())
        }
        else {
            return None;
        }
    }
    else if value.is_f64() {
        return Some(value.as_f64().unwrap().to_string())
    }
    else if value.is_i64() {
        return Some(value.as_i64().unwrap().to_string())
    }
    return None
}
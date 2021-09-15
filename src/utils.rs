use serde_json::Value;

pub fn json_number_to_str(value: &Value) -> Option<String> {
    if value.is_f64() {
        Some(value.as_f64().unwrap().to_string())
    }
    else if value.is_i64() {
        Some(value.as_i64().unwrap().to_string())
    }
    else if value.is_u64() {
        Some(value.as_u64().unwrap().to_string())
    }
    else {
        None
    }
}

pub fn json_value_to_str(value: &Value) -> Option<String> {
    if value.is_string() {
        let value_str = value.as_str().unwrap().to_lowercase();
        //We're testing for special strings
        //such as "ok" and "error" to convert them into numerical values
        //so we can use those as a gauge value
        if value_str == "ok" {
            return Some("1".into())
        }
        else if value == "error" {
            return Some("0".into())
        }
        else {
            return Some(value.to_string())
        }
    }
    else if value.is_number() {
        return json_number_to_str(value)
    }
    else if value.is_boolean() {
        return Some(value.as_bool().unwrap().to_string())
    }
    return None
}
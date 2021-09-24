use serde_json::Value;

pub fn json_number_to_i64(value: &Value) -> Option<i64> {
    //TODO: Enhance support for other numeric types
    //such as floats as well as unsigned integers
    if value.is_i64() || value.is_u64() {
        Some(value.as_i64().unwrap())
    }
    else {
        None
    }
}

pub fn json_value_to_str(value: &Value) -> Option<String> {
    if value.is_string() {
        return value.as_str().and_then(|str| Some(str.to_string()))
    }
    else if value.is_number() {
        return json_number_to_i64(value).and_then(|num| Some(num.to_string()))
    }
    else if value.is_boolean() {
        return value.as_bool().and_then(|f| Some(f.to_string()))
    }
    return None
}

pub fn json_value_to_i64(value: &Value) -> Option<i64> {
    if value.is_string() {
        let value_str = value.as_str().unwrap().to_lowercase();
        //We're testing for special strings
        //such as "ok" and "error" to convert them into numerical values
        //so we can use those as a gauge value
        if value_str == "ok" {
            return Some(1)
        }
        else if value == "error" {
            return Some(0)
        }
        else {
            return None
        }
    }
    else if value.is_number() {
        return value.as_i64().and_then(|v| Some(v))
    }
    else if value.is_boolean() {
        return value.as_bool().and_then(|f| Some(f as i64))
    }
    return None
}
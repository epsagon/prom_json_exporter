#[derive(Debug, Clone)]
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

impl ToString for PromLabel {
    fn to_string(&self) -> String {
        if self.value.parse::<i64>().is_ok() || self.value.parse::<f64>().is_ok() || self.value.parse::<bool>().is_ok() {
            format!("{}={}", self.name, self.value).to_string()
        } else {
            format!("{}=\"{}\"", self.name, self.value).to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PromLabel;

    #[test]
    fn test_string_value() {
        let label = PromLabel::new("foo".to_string(), "Foo".to_string());
        assert_eq!(format!("{}=\"{}\"","foo", "Foo"), label.to_string());
    }

    #[test]
    fn test_int_number_value() {
        let label = PromLabel::new("foo".to_string(), "34".to_string());
        assert_eq!(format!("{}={}","foo", "34"), label.to_string());
    }

    #[test]
    fn test_float_number_value() {
        let label = PromLabel::new("foo".to_string(), "3.14".to_string());
        assert_eq!(format!("{}={}","foo", "3.14"), label.to_string());
    }

    #[test]
    fn test_bool_value() {
        let label = PromLabel::new("foo".to_string(), false.to_string());
        assert_eq!(format!("{}={}","foo", "false"), label.to_string());
    }
}
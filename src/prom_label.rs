
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

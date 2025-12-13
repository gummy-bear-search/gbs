use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Index {
    pub name: String,
    pub settings: Option<serde_json::Value>,
    pub mappings: Option<serde_json::Value>,
    pub documents: HashMap<String, serde_json::Value>,
    pub aliases: Vec<String>, // List of alias names for this index
}

impl Index {
    pub fn new(name: String, settings: Option<serde_json::Value>, mappings: Option<serde_json::Value>) -> Self {
        Self {
            name,
            settings,
            mappings,
            documents: HashMap::new(),
            aliases: Vec::new(),
        }
    }
}

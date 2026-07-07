use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AIDecompilation {
    pub function_name: String,
    pub result: String,
    pub timestamp: String,
    pub model: String,
}

impl AIDecompilation {
    pub fn new(function_name: String, result: String, model: String) -> Self {
        Self {
            function_name,
            result,
            timestamp: chrono::Local::now().to_rfc3339(),
            model,
        }
    }
}
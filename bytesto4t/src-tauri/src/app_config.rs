use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub file_path: String,
    pub theme: Option<String>,
    pub colorscheme: Option<String>,
    pub recent_files: Option<Vec<String>>,
}

impl AppConfig {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            theme: None,
            colorscheme: None,
            recent_files: None,
        }
    }
}

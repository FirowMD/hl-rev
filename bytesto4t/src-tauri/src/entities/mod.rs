use hlbc::Bytecode;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub(crate) struct AppConfig {
    pub(crate) file_path: String,
    pub(crate) theme: Option<String>,
    pub(crate) colorscheme: Option<String>,
    pub(crate) recent_files: Option<Vec<String>>,
}

pub(crate) struct AppData {
    pub(crate) target_file_path: String,
    pub(crate) bytecode: Option<Bytecode>,
    pub(crate) app_config: AppConfig,
}

pub(crate) struct Storage {
    pub(crate) app_data: Mutex<AppData>,
}

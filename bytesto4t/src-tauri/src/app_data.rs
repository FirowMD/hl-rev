use crate::app_config::AppConfig;
use hlbc::Bytecode;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppItem {
    pub index: String,
    pub typ: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryItem {
    pub name: String,
    pub typ: String,
    pub timestamp: String,
}

#[derive(Debug)]
pub struct Reference {
    pub element_index: usize,
    pub references: Vec<String>,
}

pub struct BytecodeStore {
    pub target_file_path: String,
    pub bytecode: Option<Bytecode>,
    pub function_addresses: Option<Vec<String>>,
}

pub struct UiStore {
    #[allow(dead_code)]
    pub selected_item: Option<AppItem>,
}

pub struct Storage {
    pub bytecode: Mutex<BytecodeStore>,
    pub config: Mutex<AppConfig>,
    pub ui: Mutex<UiStore>,
    pub history: Mutex<Vec<HistoryItem>>,
    pub references: Mutex<Option<Reference>>,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            bytecode: Mutex::new(BytecodeStore {
                target_file_path: String::new(),
                bytecode: None,
                function_addresses: None,
            }),
            config: Mutex::new(AppConfig::new(String::new())),
            ui: Mutex::new(UiStore {
                selected_item: None,
            }),
            history: Mutex::new(Vec::new()),
            references: Mutex::new(None),
        }
    }
}

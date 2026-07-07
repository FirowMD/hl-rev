use std::sync::Mutex;
use std::collections::HashMap;
use hlbc::Bytecode;
use crate::app_config::AppConfig;
use crate::ai_decomp::AIDecompilation;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppItem {
    pub index: String,
    pub typ: String
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
    pub references: Vec<String>
}

pub struct AppData {
    pub target_file_path: String,
    pub bytecode: Option<Bytecode>,
    pub app_config: AppConfig,
    pub ai_decompilations: Option<HashMap<String, AIDecompilation>>,
    #[allow(dead_code)]
    pub selected_item: Option<AppItem>,
    pub function_addresses: Option<Vec<String>>,
    pub history_items: Mutex<Vec<HistoryItem>>,
    pub references: Option<Reference>,
}

pub struct Storage {
    pub app_data: Mutex<AppData>,
}
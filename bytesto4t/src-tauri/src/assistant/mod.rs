pub mod auth;
pub mod client;
pub(crate) mod external_http;
pub mod history;
pub(crate) mod network;
mod redaction;
mod tools;

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64},
    Mutex,
};

pub struct AssistantState {
    pub generation: AtomicU64,
    pub external_network_required: AtomicBool,
    pub model_instructions: Mutex<HashMap<String, String>>,
}

impl Default for AssistantState {
    fn default() -> Self {
        Self {
            generation: AtomicU64::new(0),
            external_network_required: AtomicBool::new(false),
            model_instructions: Mutex::new(HashMap::new()),
        }
    }
}

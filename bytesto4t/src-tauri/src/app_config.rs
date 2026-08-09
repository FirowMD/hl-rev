use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AssistantConfig {
    pub model: String,
    pub reasoning_effort: String,
    pub bypass_proxy: bool,
    pub disable_tls: bool,
    pub proxy_url: String,
    pub privacy_disclosure_version: u32,
}

impl Default for AssistantConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            reasoning_effort: "medium".to_string(),
            bypass_proxy: false,
            disable_tls: false,
            proxy_url: String::new(),
            privacy_disclosure_version: 0,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub file_path: String,
    pub theme: Option<String>,
    pub colorscheme: Option<String>,
    pub recent_files: Option<Vec<String>>,
    #[serde(default)]
    pub assistant: AssistantConfig,
}

impl AppConfig {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            theme: None,
            colorscheme: None,
            recent_files: None,
            assistant: AssistantConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn older_config_without_assistant_settings_still_loads() {
        let config: AppConfig = serde_json::from_str(
            r#"{"file_path":"config.json","theme":"dark","colorscheme":"bytesto4t","recent_files":[]}"#,
        )
        .unwrap();
        assert_eq!(config.assistant.reasoning_effort, "medium");
        assert!(!config.assistant.bypass_proxy);
        assert_eq!(config.assistant.privacy_disclosure_version, 0);
    }
}

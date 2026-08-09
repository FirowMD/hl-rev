use crate::app_config::AssistantConfig;
use crate::app_data::Storage;
use crate::assistant::{auth, client, history, network, AssistantState};
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{ipc::Channel, AppHandle, Manager, State};

pub const PRIVACY_DISCLOSURE_VERSION: u32 = 1;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantStatus {
    authenticated: bool,
    settings: AssistantConfig,
    models: Vec<String>,
    privacy_disclosure_accepted: bool,
    privacy_disclosure_version: u32,
}

fn current_settings(app_handle: &AppHandle) -> Result<AssistantConfig, String> {
    let storage = app_handle.state::<Storage>();
    let config = storage.config.lock().map_err(|error| error.to_string())?;
    Ok(config.assistant.clone())
}

fn status(app_handle: &AppHandle, models: Vec<String>) -> Result<AssistantStatus, String> {
    let settings = current_settings(app_handle)?;
    Ok(AssistantStatus {
        authenticated: auth::load_credentials()?.is_some(),
        privacy_disclosure_accepted: privacy_accepted(&settings),
        privacy_disclosure_version: PRIVACY_DISCLOSURE_VERSION,
        settings,
        models,
    })
}

fn privacy_accepted(settings: &AssistantConfig) -> bool {
    settings.privacy_disclosure_version == PRIVACY_DISCLOSURE_VERSION
}

fn require_privacy_disclosure(settings: &AssistantConfig) -> Result<(), String> {
    if privacy_accepted(settings) {
        Ok(())
    } else {
        Err("Review and accept the current Assistant privacy disclosure before connecting or sending data."
            .to_string())
    }
}

#[tauri::command]
pub fn get_assistant_status(app_handle: AppHandle) -> Result<AssistantStatus, String> {
    status(&app_handle, Vec::new())
}

#[tauri::command]
pub async fn start_chatgpt_oauth(
    app_handle: AppHandle,
    assistant_state: State<'_, AssistantState>,
) -> Result<AssistantStatus, String> {
    let settings = current_settings(&app_handle)?;
    require_privacy_disclosure(&settings)?;
    auth::authenticate(&settings).await?;
    let models = client::discover_models(&settings, assistant_state.inner()).await?;
    status(&app_handle, models)
}

#[tauri::command]
pub fn disconnect_chatgpt(app_handle: AppHandle) -> Result<AssistantStatus, String> {
    auth::delete_credentials()?;
    status(&app_handle, Vec::new())
}

#[tauri::command]
pub async fn discover_chatgpt_models(
    app_handle: AppHandle,
    assistant_state: State<'_, AssistantState>,
) -> Result<Vec<String>, String> {
    let settings = current_settings(&app_handle)?;
    require_privacy_disclosure(&settings)?;
    client::discover_models(&settings, assistant_state.inner()).await
}

#[tauri::command]
pub fn accept_assistant_privacy_disclosure(
    app_handle: AppHandle,
    version: u32,
) -> Result<(), String> {
    if version != PRIVACY_DISCLOSURE_VERSION {
        return Err("The Assistant privacy disclosure version is not current.".to_string());
    }
    {
        let storage = app_handle.state::<Storage>();
        let mut config = storage.config.lock().map_err(|error| error.to_string())?;
        config.assistant.privacy_disclosure_version = version;
    }
    crate::commands::config::save_config(app_handle)
}

#[tauri::command]
pub fn update_assistant_settings(
    app_handle: AppHandle,
    assistant_state: State<'_, AssistantState>,
    mut settings: AssistantConfig,
    confirm_tls_bypass: bool,
) -> Result<(), String> {
    validate_settings(&settings)?;
    {
        let storage = app_handle.state::<Storage>();
        let mut config = storage.config.lock().map_err(|error| error.to_string())?;
        if settings.disable_tls && !config.assistant.disable_tls && !confirm_tls_bypass {
            return Err(
                "Enabling TLS-verification bypass requires explicit confirmation.".to_string(),
            );
        }
        settings.privacy_disclosure_version = config.assistant.privacy_disclosure_version;
        config.assistant = settings;
    }
    assistant_state
        .external_network_required
        .store(false, Ordering::SeqCst);
    crate::commands::config::save_config(app_handle)
}

fn validate_settings(settings: &AssistantConfig) -> Result<(), String> {
    if !matches!(
        settings.reasoning_effort.as_str(),
        "none" | "low" | "medium" | "high" | "xhigh"
    ) {
        return Err("Unsupported reasoning effort.".to_string());
    }
    network::validate_proxy_url(&settings.proxy_url)
}

#[tauri::command]
pub fn load_assistant_chats(
    app_handle: AppHandle,
) -> Result<Option<history::AssistantChatStore>, String> {
    history::load(&app_handle)
}

#[tauri::command]
pub fn save_assistant_chats(
    app_handle: AppHandle,
    store: history::AssistantChatStore,
) -> Result<(), String> {
    history::save(&app_handle, &store)
}

#[tauri::command]
pub fn delete_assistant_chats(app_handle: AppHandle) -> Result<(), String> {
    history::delete(&app_handle)
}

#[tauri::command]
pub fn migrate_assistant_chats(
    app_handle: AppHandle,
    v2_json: Option<String>,
    v1_json: Option<String>,
) -> Result<history::AssistantChatStore, String> {
    history::migrate(&app_handle, v2_json.as_deref(), v1_json.as_deref())
}

#[tauri::command]
pub async fn send_assistant_message(
    app_handle: AppHandle,
    assistant_state: State<'_, AssistantState>,
    request: client::AssistantChatRequest,
    on_event: Channel<client::AssistantStreamEvent>,
) -> Result<client::AssistantReply, String> {
    let settings = current_settings(&app_handle)?;
    require_privacy_disclosure(&settings)?;
    client::chat(
        app_handle,
        assistant_state.inner(),
        settings,
        request,
        on_event,
    )
    .await
}

#[tauri::command]
pub fn cancel_assistant_message(assistant_state: State<'_, AssistantState>) {
    assistant_state.generation.fetch_add(1, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_acceptance_is_tied_to_current_version() {
        let mut settings = AssistantConfig::default();
        assert!(!privacy_accepted(&settings));
        settings.privacy_disclosure_version = PRIVACY_DISCLOSURE_VERSION;
        assert!(privacy_accepted(&settings));
        settings.privacy_disclosure_version = PRIVACY_DISCLOSURE_VERSION + 1;
        assert!(!privacy_accepted(&settings));
    }
}

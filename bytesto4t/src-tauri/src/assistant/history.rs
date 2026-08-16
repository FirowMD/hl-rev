use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};
use zeroize::Zeroizing;

const HISTORY_SERVICE: &str = "com.firowmd.bytesto4t.assistant-history";
const HISTORY_KEY_USER: &str = "encryption-key.v1";
const HISTORY_FILE: &str = "assistant-chats.v1.json";
const HISTORY_FORMAT_VERSION: u8 = 1;
const HISTORY_ALGORITHM: &str = "XChaCha20-Poly1305";
const HISTORY_AAD: &[u8] = b"bytesto4t.assistant.chats.v1";
const MAX_PLAINTEXT_BYTES: usize = 8 * 1024 * 1024;
const MAX_ENVELOPE_BYTES: u64 = 12 * 1024 * 1024;
const MAX_CHATS: usize = 100;
const MAX_MESSAGES_PER_CHAT: usize = 100;
const MAX_MESSAGE_BYTES: usize = 500_000;
static HISTORY_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssistantChatStore {
    pub active_chat_id: String,
    pub chats: Vec<StoredChat>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredChat {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub messages: Vec<StoredChatMessage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_activity: Option<Vec<StoredToolActivity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<StoredUsage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredToolActivity {
    pub name: String,
    pub arguments: Value,
    pub success: bool,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EncryptedEnvelope {
    version: u8,
    algorithm: String,
    nonce: String,
    ciphertext: String,
}

fn io_error(context: &str, error: &std::io::Error) -> String {
    format!("{context} ({:?}).", error.kind())
}

fn history_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    app_handle
        .path()
        .app_data_dir()
        .map(|directory| directory.join(HISTORY_FILE))
        .map_err(|_| "Could not resolve the application data directory.".to_string())
}

fn history_key_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(HISTORY_SERVICE, HISTORY_KEY_USER).map_err(|_| {
        "Could not open the chat-history key in the system credential vault.".to_string()
    })
}

fn history_key(create: bool) -> Result<Zeroizing<Vec<u8>>, String> {
    let entry = history_key_entry()?;
    let encoded = match entry.get_password() {
        Ok(value) => Zeroizing::new(value),
        Err(keyring::Error::NoEntry) if create => {
            let mut key = Zeroizing::new(vec![0_u8; 32]);
            rand::rng().fill_bytes(&mut key);
            let encoded = Zeroizing::new(URL_SAFE_NO_PAD.encode(&*key));
            entry.set_password(&encoded).map_err(|_| {
                "Could not store the chat-history key in the system credential vault.".to_string()
            })?;
            return Ok(key);
        }
        Err(keyring::Error::NoEntry) => {
            return Err(
                "Encrypted chat history exists, but its credential-vault key is unavailable."
                    .to_string(),
            )
        }
        Err(_) => {
            return Err(
                "Could not read the chat-history key from the system credential vault.".to_string(),
            )
        }
    };
    let key = URL_SAFE_NO_PAD.decode(encoded.as_bytes()).map_err(|_| {
        "The chat-history key in the system credential vault is invalid.".to_string()
    })?;
    if key.len() != 32 {
        return Err("The chat-history key in the system credential vault is invalid.".to_string());
    }
    Ok(Zeroizing::new(key))
}

fn validate_store(store: &AssistantChatStore) -> Result<(), String> {
    if store.chats.len() > MAX_CHATS {
        return Err("Assistant chat history contains too many chats.".to_string());
    }
    if store.chats.is_empty() {
        if !store.active_chat_id.is_empty() {
            return Err("Empty Assistant chat history has an invalid active chat.".to_string());
        }
        return Ok(());
    }
    let mut ids = HashSet::with_capacity(store.chats.len());
    for chat in &store.chats {
        if chat.id.is_empty()
            || chat.id.len() > 128
            || chat.id.chars().any(char::is_control)
            || !ids.insert(chat.id.as_str())
        {
            return Err("Assistant chat history contains an invalid chat ID.".to_string());
        }
        if chat.title.len() > 200 || chat.title.chars().any(char::is_control) {
            return Err("Assistant chat history contains an invalid title.".to_string());
        }
        if chat.created_at > chat.updated_at || chat.messages.len() > MAX_MESSAGES_PER_CHAT {
            return Err("Assistant chat history contains invalid chat metadata.".to_string());
        }
        for message in &chat.messages {
            if !matches!(message.role.as_str(), "user" | "assistant")
                || message.content.len() > MAX_MESSAGE_BYTES
                || message
                    .model
                    .as_ref()
                    .is_some_and(|model| model.len() > 200)
                || message
                    .reasoning
                    .as_ref()
                    .is_some_and(|parts| parts.len() > 24)
                || message
                    .tool_activity
                    .as_ref()
                    .is_some_and(|activities| activities.len() > 64)
            {
                return Err("Assistant chat history contains an invalid message.".to_string());
            }
        }
    }
    if !ids.contains(store.active_chat_id.as_str()) {
        return Err("Assistant chat history has an invalid active chat.".to_string());
    }
    Ok(())
}

fn encrypt_store(store: &AssistantChatStore, key: &[u8]) -> Result<Vec<u8>, String> {
    validate_store(store)?;
    let plaintext = Zeroizing::new(
        serde_json::to_vec(store)
            .map_err(|_| "Could not serialize Assistant chat history.".to_string())?,
    );
    if plaintext.len() > MAX_PLAINTEXT_BYTES {
        return Err("Assistant chat history exceeds the encrypted storage limit.".to_string());
    }
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| "The chat-history encryption key is invalid.".to_string())?;
    let mut nonce = [0_u8; 24];
    rand::rng().fill_bytes(&mut nonce);
    let cipher_nonce = XNonce::from(nonce);
    let ciphertext = cipher
        .encrypt(
            &cipher_nonce,
            Payload {
                msg: &plaintext,
                aad: HISTORY_AAD,
            },
        )
        .map_err(|_| "Could not encrypt Assistant chat history.".to_string())?;
    let envelope = EncryptedEnvelope {
        version: HISTORY_FORMAT_VERSION,
        algorithm: HISTORY_ALGORITHM.to_string(),
        nonce: URL_SAFE_NO_PAD.encode(nonce),
        ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
    };
    serde_json::to_vec(&envelope)
        .map_err(|_| "Could not serialize encrypted Assistant chat history.".to_string())
}

fn decrypt_store(envelope_bytes: &[u8], key: &[u8]) -> Result<AssistantChatStore, String> {
    if envelope_bytes.len() as u64 > MAX_ENVELOPE_BYTES {
        return Err("Encrypted Assistant chat history exceeds the storage limit.".to_string());
    }
    let envelope: EncryptedEnvelope = serde_json::from_slice(envelope_bytes)
        .map_err(|_| "Encrypted Assistant chat history is invalid.".to_string())?;
    if envelope.version != HISTORY_FORMAT_VERSION || envelope.algorithm != HISTORY_ALGORITHM {
        return Err("Encrypted Assistant chat history uses an unsupported format.".to_string());
    }
    let nonce = URL_SAFE_NO_PAD
        .decode(envelope.nonce)
        .map_err(|_| "Encrypted Assistant chat history has an invalid nonce.".to_string())?;
    if nonce.len() != 24 {
        return Err("Encrypted Assistant chat history has an invalid nonce.".to_string());
    }
    let nonce = XNonce::try_from(nonce.as_slice())
        .map_err(|_| "Encrypted Assistant chat history has an invalid nonce.".to_string())?;
    let ciphertext = URL_SAFE_NO_PAD
        .decode(envelope.ciphertext)
        .map_err(|_| "Encrypted Assistant chat history has invalid ciphertext.".to_string())?;
    if ciphertext.len() > MAX_PLAINTEXT_BYTES + 16 {
        return Err("Encrypted Assistant chat history exceeds the storage limit.".to_string());
    }
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| "The chat-history encryption key is invalid.".to_string())?;
    let plaintext = Zeroizing::new(
        cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &ciphertext,
                    aad: HISTORY_AAD,
                },
            )
            .map_err(|_| "Encrypted Assistant chat history failed authentication.".to_string())?,
    );
    if plaintext.len() > MAX_PLAINTEXT_BYTES {
        return Err("Assistant chat history exceeds the encrypted storage limit.".to_string());
    }
    let store: AssistantChatStore = serde_json::from_slice(&plaintext)
        .map_err(|_| "Decrypted Assistant chat history is invalid.".to_string())?;
    validate_store(&store)?;
    Ok(store)
}

fn atomic_replace(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Assistant chat-history path is invalid.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| io_error("Could not create the Assistant data directory", &error))?;
    let temporary = parent.join(format!(".{HISTORY_FILE}.{}.tmp", random_id()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|error| io_error("Could not create a temporary chat-history file", &error))?;
        file.write_all(contents).map_err(|error| {
            io_error("Could not write encrypted Assistant chat history", &error)
        })?;
        file.sync_all().map_err(|error| {
            io_error("Could not flush encrypted Assistant chat history", &error)
        })?;
        fs::rename(&temporary, path).map_err(|error| {
            io_error(
                "Could not atomically replace Assistant chat history",
                &error,
            )
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn read_envelope(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(io_error(
                "Could not open encrypted Assistant chat history",
                &error,
            ))
        }
    };
    let length = file
        .metadata()
        .map_err(|error| io_error("Could not inspect encrypted Assistant chat history", &error))?
        .len();
    if length > MAX_ENVELOPE_BYTES {
        return Err("Encrypted Assistant chat history exceeds the storage limit.".to_string());
    }
    let mut contents = Vec::with_capacity(length as usize);
    file.read_to_end(&mut contents)
        .map_err(|error| io_error("Could not read encrypted Assistant chat history", &error))?;
    Ok(Some(contents))
}

fn save_at_path(path: &Path, store: &AssistantChatStore, key: &[u8]) -> Result<(), String> {
    if store.chats.is_empty() {
        return delete_at_path(path);
    }
    let envelope = encrypt_store(store, key)?;
    atomic_replace(path, &envelope)
}

fn load_at_path(path: &Path, key: &[u8]) -> Result<Option<AssistantChatStore>, String> {
    read_envelope(path)?
        .map(|contents| decrypt_store(&contents, key))
        .transpose()
}

fn delete_at_path(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(
            "Could not remove encrypted Assistant chat history",
            &error,
        )),
    }
}

fn random_id() -> String {
    let mut bytes = [0_u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn migrated_title(messages: &[StoredChatMessage]) -> String {
    let content = messages
        .iter()
        .find(|message| message.role == "user")
        .map(|message| {
            message
                .content
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    if content.is_empty() {
        "Migrated chat".to_string()
    } else if content.chars().count() > 42 {
        format!("{}...", content.chars().take(39).collect::<String>())
    } else {
        content
    }
}

fn parse_migration(
    v2_json: Option<&str>,
    v1_json: Option<&str>,
) -> Result<AssistantChatStore, String> {
    if let Some(value) = v2_json {
        if value.len() > MAX_PLAINTEXT_BYTES {
            return Err(
                "Plaintext Assistant chat history exceeds the migration limit.".to_string(),
            );
        }
        let store: AssistantChatStore = serde_json::from_str(value)
            .map_err(|_| "Plaintext Assistant v2 chat history is invalid.".to_string())?;
        validate_store(&store)?;
        return Ok(store);
    }
    let value =
        v1_json.ok_or_else(|| "No plaintext Assistant chat history was supplied.".to_string())?;
    if value.len() > MAX_PLAINTEXT_BYTES {
        return Err("Plaintext Assistant chat history exceeds the migration limit.".to_string());
    }
    let messages: Vec<StoredChatMessage> = serde_json::from_str(value)
        .map_err(|_| "Plaintext Assistant v1 chat history is invalid.".to_string())?;
    if messages.is_empty() {
        return Ok(AssistantChatStore {
            active_chat_id: String::new(),
            chats: Vec::new(),
        });
    }
    let timestamp = now_millis();
    let id = format!("migrated-{}", random_id());
    let store = AssistantChatStore {
        active_chat_id: id.clone(),
        chats: vec![StoredChat {
            id,
            title: migrated_title(&messages),
            created_at: timestamp,
            updated_at: timestamp,
            messages,
        }],
    };
    validate_store(&store)?;
    Ok(store)
}

pub fn load(app_handle: &AppHandle) -> Result<Option<AssistantChatStore>, String> {
    let _guard = HISTORY_LOCK
        .lock()
        .map_err(|_| "Assistant chat-history lock is unavailable.".to_string())?;
    let path = history_path(app_handle)?;
    if !path.exists() {
        return Ok(None);
    }
    let key = history_key(false)?;
    load_at_path(&path, &key)
}

pub fn save(app_handle: &AppHandle, store: &AssistantChatStore) -> Result<(), String> {
    let _guard = HISTORY_LOCK
        .lock()
        .map_err(|_| "Assistant chat-history lock is unavailable.".to_string())?;
    let path = history_path(app_handle)?;
    if store.chats.is_empty() {
        return delete_at_path(&path);
    }
    let key = history_key(true)?;
    save_at_path(&path, store, &key)
}

pub fn delete(app_handle: &AppHandle) -> Result<(), String> {
    let _guard = HISTORY_LOCK
        .lock()
        .map_err(|_| "Assistant chat-history lock is unavailable.".to_string())?;
    delete_at_path(&history_path(app_handle)?)
}

pub fn migrate(
    app_handle: &AppHandle,
    v2_json: Option<&str>,
    v1_json: Option<&str>,
) -> Result<AssistantChatStore, String> {
    let _guard = HISTORY_LOCK
        .lock()
        .map_err(|_| "Assistant chat-history lock is unavailable.".to_string())?;
    let path = history_path(app_handle)?;
    if path.exists() {
        let key = history_key(false)?;
        return load_at_path(&path, &key)?.ok_or_else(|| {
            "Encrypted Assistant chat history disappeared during migration.".to_string()
        });
    }
    let store = parse_migration(v2_json, v1_json)?;
    if !store.chats.is_empty() {
        let key = history_key(true)?;
        save_at_path(&path, &store, &key)?;
    }
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_store(content: &str) -> AssistantChatStore {
        AssistantChatStore {
            active_chat_id: "chat-1".to_string(),
            chats: vec![StoredChat {
                id: "chat-1".to_string(),
                title: "Test".to_string(),
                created_at: 1,
                updated_at: 2,
                messages: vec![StoredChatMessage {
                    role: "user".to_string(),
                    content: content.to_string(),
                    model: None,
                    reasoning: None,
                    tool_activity: None,
                    usage: None,
                }],
            }],
        }
    }

    #[test]
    fn encrypted_history_round_trips_without_plaintext() {
        let store = sample_store("private reverse-engineering prompt");
        let key = [7_u8; 32];
        let envelope = encrypt_store(&store, &key).unwrap();
        assert!(!String::from_utf8_lossy(&envelope).contains("private reverse-engineering prompt"));
        let restored = decrypt_store(&envelope, &key).unwrap();
        assert_eq!(
            restored.chats[0].messages[0].content,
            store.chats[0].messages[0].content
        );
    }

    #[test]
    fn tampering_and_unknown_versions_fail_closed() {
        let store = sample_store("content");
        let key = [9_u8; 32];
        let envelope = encrypt_store(&store, &key).unwrap();
        let mut value: Value = serde_json::from_slice(&envelope).unwrap();
        value["ciphertext"] = Value::String(URL_SAFE_NO_PAD.encode(b"tampered"));
        assert!(decrypt_store(&serde_json::to_vec(&value).unwrap(), &key).is_err());
        value["version"] = Value::from(99);
        assert!(decrypt_store(&serde_json::to_vec(&value).unwrap(), &key).is_err());
    }

    #[test]
    fn legacy_v1_history_migrates_to_encrypted_storage() {
        let legacy = r#"[{"role":"user","content":"legacy prompt"}]"#;
        let store = parse_migration(None, Some(legacy)).unwrap();
        assert_eq!(store.chats.len(), 1);
        assert_eq!(store.chats[0].title, "legacy prompt");

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(HISTORY_FILE);
        let key = [11_u8; 32];
        save_at_path(&path, &store, &key).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("legacy prompt"));
        assert_eq!(load_at_path(&path, &key).unwrap().unwrap().chats.len(), 1);
    }

    #[test]
    fn atomic_replacement_overwrites_previous_history() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(HISTORY_FILE);
        let key = [13_u8; 32];
        save_at_path(&path, &sample_store("first"), &key).unwrap();
        save_at_path(&path, &sample_store("second"), &key).unwrap();
        let restored = load_at_path(&path, &key).unwrap().unwrap();
        assert_eq!(restored.chats[0].messages[0].content, "second");
    }

    #[test]
    fn removal_deletes_encrypted_history_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(HISTORY_FILE);
        let key = [15_u8; 32];
        save_at_path(&path, &sample_store("delete me"), &key).unwrap();
        delete_at_path(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn oversized_history_is_rejected() {
        let store = sample_store(&"x".repeat(MAX_MESSAGE_BYTES + 1));
        assert!(encrypt_store(&store, &[17_u8; 32]).is_err());
    }
}

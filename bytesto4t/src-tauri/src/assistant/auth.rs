use super::external_http;
use super::network::{build_client, is_access_denied, network_error};
use super::redaction::redact;
use crate::app_config::AssistantConfig;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use url::Url;

const OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const OAUTH_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const OAUTH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const OAUTH_CALLBACK_PATH: &str = "/auth/callback";
const OAUTH_CALLBACK_PORTS: &[u16] = &[1455, 1456, 1457, 1458, 1459, 1460];
const OAUTH_SCOPES: &str = "openid profile email offline_access";
const CREDENTIAL_SERVICE: &str = "com.firowmd.bytesto4t.chatgpt";
const CREDENTIAL_USER: &str = "oauth_tokens";
const CREDENTIAL_CHUNK_PREFIX: &str = "oauth_tokens.chunk";
const CREDENTIAL_FORMAT_VERSION: u8 = 1;
// Windows Credential Manager allows a 2560-byte credential blob. Passwords are
// stored as UTF-16 by keyring, so keep each ASCII/base64 chunk well below that.
const CREDENTIAL_CHUNK_CHARS: usize = 1024;
const MAX_CREDENTIAL_CHUNKS: usize = 128;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub account_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CredentialManifest {
    version: u8,
    generation: String,
    chunks: usize,
    checksum: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default = "default_expires_in")]
    expires_in: u64,
    #[serde(default)]
    id_token: String,
}

fn default_expires_in() -> u64 {
    3600
}

fn credential_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(CREDENTIAL_SERVICE, CREDENTIAL_USER)
        .map_err(|error| format!("Could not open the system credential vault: {error}"))
}

fn credential_chunk_entry(generation: &str, index: usize) -> Result<keyring::Entry, String> {
    keyring::Entry::new(
        CREDENTIAL_SERVICE,
        &format!("{CREDENTIAL_CHUNK_PREFIX}.{generation}.{index}"),
    )
    .map_err(|error| format!("Could not open the system credential vault: {error}"))
}

fn validate_manifest(manifest: &CredentialManifest) -> Result<(), String> {
    if manifest.version != CREDENTIAL_FORMAT_VERSION {
        return Err(format!(
            "Stored ChatGPT credentials use unsupported format version {}.",
            manifest.version
        ));
    }
    if manifest.chunks == 0 || manifest.chunks > MAX_CREDENTIAL_CHUNKS {
        return Err("Stored ChatGPT credentials have an invalid chunk count.".to_string());
    }
    if manifest.generation.is_empty()
        || manifest.generation.len() > 64
        || !manifest
            .generation
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("Stored ChatGPT credentials have an invalid generation ID.".to_string());
    }
    Ok(())
}

fn encode_credential_chunks(
    credentials: &Credentials,
) -> Result<(CredentialManifest, Vec<String>), String> {
    let serialized = serde_json::to_vec(credentials)
        .map_err(|error| format!("Could not encode ChatGPT credentials: {error}"))?;
    let encoded = URL_SAFE_NO_PAD.encode(serialized);
    let chunks: Vec<String> = encoded
        .as_bytes()
        .chunks(CREDENTIAL_CHUNK_CHARS)
        .map(|chunk| String::from_utf8(chunk.to_vec()).expect("base64 is valid UTF-8"))
        .collect();
    if chunks.is_empty() || chunks.len() > MAX_CREDENTIAL_CHUNKS {
        return Err("ChatGPT credentials are too large for secure storage.".to_string());
    }
    let manifest = CredentialManifest {
        version: CREDENTIAL_FORMAT_VERSION,
        generation: random_urlsafe(12),
        chunks: chunks.len(),
        checksum: URL_SAFE_NO_PAD.encode(Sha256::digest(encoded.as_bytes())),
    };
    Ok((manifest, chunks))
}

fn decode_credential_chunks(
    manifest: &CredentialManifest,
    chunks: &[String],
) -> Result<Credentials, String> {
    validate_manifest(manifest)?;
    if chunks.len() != manifest.chunks {
        return Err("Stored ChatGPT credentials are incomplete.".to_string());
    }
    let encoded = chunks.concat();
    let checksum = URL_SAFE_NO_PAD.encode(Sha256::digest(encoded.as_bytes()));
    if checksum != manifest.checksum {
        return Err("Stored ChatGPT credentials failed their integrity check.".to_string());
    }
    let serialized = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|error| format!("Stored ChatGPT credentials are invalid: {error}"))?;
    serde_json::from_slice(&serialized)
        .map_err(|error| format!("Stored ChatGPT credentials are invalid: {error}"))
}

fn delete_entry(entry: keyring::Entry) -> Result<(), String> {
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

trait CredentialVault {
    fn read(&self, user: &str) -> Result<Option<String>, String>;
    fn delete(&self, user: &str) -> Result<(), String>;
}

struct SystemCredentialVault;

impl CredentialVault for SystemCredentialVault {
    fn read(&self, user: &str) -> Result<Option<String>, String> {
        let entry = keyring::Entry::new(CREDENTIAL_SERVICE, user)
            .map_err(|error| redact(error.to_string()))?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(redact(error.to_string())),
        }
    }

    fn delete(&self, user: &str) -> Result<(), String> {
        let entry = keyring::Entry::new(CREDENTIAL_SERVICE, user)
            .map_err(|error| redact(error.to_string()))?;
        delete_entry(entry).map_err(redact)
    }
}

fn credential_chunk_user(generation: &str, index: usize) -> String {
    format!("{CREDENTIAL_CHUNK_PREFIX}.{generation}.{index}")
}

fn delete_credentials_from(vault: &impl CredentialVault) -> Result<(), String> {
    let Some(value) = vault.read(CREDENTIAL_USER)? else {
        return Ok(());
    };
    if let Ok(manifest) = serde_json::from_str::<CredentialManifest>(&value) {
        validate_manifest(&manifest)?;
        for index in 0..manifest.chunks {
            vault.delete(&credential_chunk_user(&manifest.generation, index))?;
        }
    }
    vault.delete(CREDENTIAL_USER)
}

fn delete_manifest_chunks(manifest: &CredentialManifest) -> Result<(), String> {
    validate_manifest(manifest)?;
    for index in 0..manifest.chunks {
        delete_entry(credential_chunk_entry(&manifest.generation, index)?)?;
    }
    Ok(())
}

pub fn load_credentials() -> Result<Option<Credentials>, String> {
    match credential_entry()?.get_password() {
        Ok(value) => {
            if let Ok(manifest) = serde_json::from_str::<CredentialManifest>(&value) {
                validate_manifest(&manifest)?;
                let mut chunks = Vec::with_capacity(manifest.chunks);
                for index in 0..manifest.chunks {
                    let chunk = credential_chunk_entry(&manifest.generation, index)?
                        .get_password()
                        .map_err(|error| {
                            format!("Could not read ChatGPT credential chunk {index}: {error}")
                        })?;
                    chunks.push(chunk);
                }
                return decode_credential_chunks(&manifest, &chunks).map(Some);
            }

            // Migrate credentials written by the original single-entry format.
            let credentials: Credentials = serde_json::from_str(&value)
                .map_err(|error| format!("Stored ChatGPT credentials are invalid: {error}"))?;
            save_credentials(&credentials)?;
            Ok(Some(credentials))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!(
            "Could not read ChatGPT credentials from the system vault: {error}"
        )),
    }
}

pub fn save_credentials(credentials: &Credentials) -> Result<(), String> {
    let previous_manifest = credential_entry()?
        .get_password()
        .ok()
        .and_then(|value| serde_json::from_str::<CredentialManifest>(&value).ok());
    let (manifest, chunks) = encode_credential_chunks(credentials)?;

    let mut written_chunks = 0;
    for (index, chunk) in chunks.iter().enumerate() {
        if let Err(error) = credential_chunk_entry(&manifest.generation, index)?.set_password(chunk)
        {
            for cleanup_index in 0..written_chunks {
                if let Ok(entry) = credential_chunk_entry(&manifest.generation, cleanup_index) {
                    let _ = delete_entry(entry);
                }
            }
            return Err(format!(
                "Could not store ChatGPT credential chunk {index} securely: {error}"
            ));
        }
        written_chunks += 1;
    }

    let manifest_value = serde_json::to_string(&manifest)
        .map_err(|error| format!("Could not encode ChatGPT credential metadata: {error}"))?;
    if let Err(error) = credential_entry()?.set_password(&manifest_value) {
        let _ = delete_manifest_chunks(&manifest);
        return Err(format!(
            "Could not store ChatGPT credential metadata securely: {error}"
        ));
    }

    if let Some(previous_manifest) = previous_manifest {
        if previous_manifest.generation != manifest.generation {
            let _ = delete_manifest_chunks(&previous_manifest);
        }
    }
    Ok(())
}

pub fn delete_credentials() -> Result<(), String> {
    delete_credentials_from(&SystemCredentialVault)
        .map_err(|error| format!("Could not remove ChatGPT credentials: {}", redact(error)))
}

pub fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn random_urlsafe(bytes: usize) -> String {
    let mut value = vec![0_u8; bytes];
    rand::rng().fill_bytes(&mut value);
    URL_SAFE_NO_PAD.encode(value)
}

pub fn generate_pkce() -> (String, String) {
    let verifier = random_urlsafe(32);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    (verifier, challenge)
}

pub fn build_authorize_url(challenge: &str, state: &str, redirect_uri: &str) -> String {
    let mut url = Url::parse(OAUTH_AUTH_URL).expect("OAuth authorization URL is valid");
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", OAUTH_CLIENT_ID)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", OAUTH_SCOPES)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state)
        .append_pair("id_token_add_organizations", "true")
        .append_pair("codex_cli_simplified_flow", "true")
        .append_pair("originator", "codex_cli_rs");
    url.to_string()
}

fn parse_jwt_claims(token: &str) -> Option<Value> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn account_id_from_claims(claims: &Value) -> Option<String> {
    claims
        .get("chatgpt_account_id")
        .and_then(Value::as_str)
        .or_else(|| {
            claims
                .get("https://api.openai.com/auth")
                .and_then(|auth| auth.get("chatgpt_account_id"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            claims
                .get("organizations")
                .and_then(Value::as_array)
                .and_then(|organizations| organizations.first())
                .and_then(|organization| organization.get("id"))
                .and_then(Value::as_str)
        })
        .map(str::to_string)
}

fn extract_account_id(tokens: &TokenResponse) -> Option<String> {
    [&tokens.id_token, &tokens.access_token]
        .into_iter()
        .filter(|token| !token.is_empty())
        .find_map(|token| {
            parse_jwt_claims(token).and_then(|claims| account_id_from_claims(&claims))
        })
}

async fn post_token_form(
    form: &[(&str, &str)],
    config: &AssistantConfig,
    context: &str,
) -> Result<(u16, String), String> {
    let client = build_client(config, 90)?;
    match client.post(OAUTH_TOKEN_URL).form(form).send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = response.text().await.map_err(|error| {
                network_error("Could not read the ChatGPT token response", error)
            })?;
            Ok((status, body))
        }
        Err(error) if is_access_denied(&error) => {
            let encoded_form = url::form_urlencoded::Serializer::new(String::new())
                .extend_pairs(form.iter().copied())
                .finish();
            let direct_error = redact(error.to_string());
            let response = external_http::request(
                "POST",
                OAUTH_TOKEN_URL,
                vec![(
                    "Content-Type".to_string(),
                    "application/x-www-form-urlencoded".to_string(),
                )],
                Some(encoded_form),
                config,
                90,
            )
            .await
            .map_err(|helper_error| {
                format!(
                    "{context}: {direct_error}. External VPN network helper failed: {helper_error}"
                )
            })?;
            Ok((response.status, response.body))
        }
        Err(error) => Err(network_error(context, error)),
    }
}

async fn exchange_code(
    code: &str,
    verifier: &str,
    redirect_uri: &str,
    config: &AssistantConfig,
) -> Result<Credentials, String> {
    let (status, body) = post_token_form(
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", OAUTH_CLIENT_ID),
            ("code_verifier", verifier),
        ],
        config,
        "ChatGPT token exchange failed",
    )
    .await?;
    if !(200..300).contains(&status) {
        return Err(format!(
            "ChatGPT token exchange failed with HTTP status {status}."
        ));
    }
    let tokens: TokenResponse = serde_json::from_str(&body)
        .map_err(|error| format!("ChatGPT returned an invalid token response: {error}"))?;
    Ok(Credentials {
        account_id: extract_account_id(&tokens),
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_at: now_epoch_seconds().saturating_add(tokens.expires_in),
    })
}

pub async fn refresh_credentials(
    credentials: &Credentials,
    config: &AssistantConfig,
) -> Result<Credentials, String> {
    if credentials.refresh_token.is_empty() {
        return Err("The ChatGPT session cannot be refreshed. Sign in again.".to_string());
    }
    let (status, body) = post_token_form(
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", credentials.refresh_token.as_str()),
            ("client_id", OAUTH_CLIENT_ID),
        ],
        config,
        "ChatGPT token refresh failed",
    )
    .await?;
    if !(200..300).contains(&status) {
        return Err(format!(
            "ChatGPT token refresh failed with HTTP status {status}."
        ));
    }
    let tokens: TokenResponse = serde_json::from_str(&body)
        .map_err(|error| format!("ChatGPT returned an invalid refresh response: {error}"))?;
    let refreshed = Credentials {
        account_id: extract_account_id(&tokens).or_else(|| credentials.account_id.clone()),
        access_token: tokens.access_token,
        refresh_token: if tokens.refresh_token.is_empty() {
            credentials.refresh_token.clone()
        } else {
            tokens.refresh_token
        },
        expires_at: now_epoch_seconds().saturating_add(tokens.expires_in),
    };
    save_credentials(&refreshed)?;
    Ok(refreshed)
}

pub async fn valid_credentials(config: &AssistantConfig) -> Result<Credentials, String> {
    let credentials = load_credentials()?
        .ok_or_else(|| "Connect a ChatGPT account before using the assistant.".to_string())?;
    if credentials.expires_at <= now_epoch_seconds().saturating_add(300) {
        refresh_credentials(&credentials, config).await
    } else {
        Ok(credentials)
    }
}

async fn bind_callback_listener() -> Result<(TcpListener, String), String> {
    for port in OAUTH_CALLBACK_PORTS {
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", *port)).await {
            return Ok((
                listener,
                format!("http://localhost:{port}{OAUTH_CALLBACK_PATH}"),
            ));
        }
    }
    Err("Could not start the ChatGPT callback server. Ports 1455-1460 are in use.".to_string())
}

async fn send_callback_page(
    stream: &mut tokio::net::TcpStream,
    success: bool,
) -> Result<(), String> {
    let (status, title, message) = if success {
        (
            "200 OK",
            "Connected to ChatGPT",
            "Authentication is complete. You can close this tab and return to bytesto4t.",
        )
    } else {
        (
            "400 Bad Request",
            "Authentication failed",
            "Return to bytesto4t for details and try again.",
        )
    };
    let body = format!(
        "<!doctype html><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>{title}</title><style>body{{margin:0;background:#111418;color:#e8edf2;font:16px system-ui;display:grid;place-items:center;min-height:100vh}}main{{max-width:34rem;padding:2rem}}h1{{font-size:1.5rem}}p{{color:#aeb8c2;line-height:1.5}}</style><main><h1>{title}</h1><p>{message}</p></main>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|error| format!("Could not answer the OAuth callback: {error}"))
}

async fn wait_for_callback(
    listener: TcpListener,
    expected_state: &str,
) -> Result<(String, tokio::net::TcpStream), String> {
    let callback = async {
        loop {
            let (mut stream, peer) = listener
                .accept()
                .await
                .map_err(|error| format!("OAuth callback failed: {error}"))?;
            if !peer.ip().is_loopback() {
                let _ = send_callback_page(&mut stream, false).await;
                continue;
            }
            let mut buffer = vec![0_u8; 16 * 1024];
            let count = stream
                .read(&mut buffer)
                .await
                .map_err(|error| format!("Could not read the OAuth callback: {error}"))?;
            let request = String::from_utf8_lossy(&buffer[..count]);
            let Some(target) = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
            else {
                let _ = send_callback_page(&mut stream, false).await;
                continue;
            };
            match callback_code(target, expected_state) {
                Ok(Some(code)) => return Ok((code, stream)),
                Ok(None) => {
                    let _ = send_callback_page(&mut stream, false).await;
                }
                Err(error) => {
                    let _ = send_callback_page(&mut stream, false).await;
                    return Err(error);
                }
            }
        }
    };

    tokio::time::timeout(std::time::Duration::from_secs(300), callback)
        .await
        .map_err(|_| "ChatGPT authentication timed out after five minutes.".to_string())?
}

fn callback_code(target: &str, expected_state: &str) -> Result<Option<String>, String> {
    let url = Url::parse(&format!("http://localhost{target}"))
        .map_err(|_| "The ChatGPT callback URL was invalid.".to_string())?;
    if url.path() != OAUTH_CALLBACK_PATH {
        return Ok(None);
    }
    let params: Vec<_> = url.query_pairs().into_owned().collect();
    let states: Vec<_> = params
        .iter()
        .filter(|(name, _)| name == "state")
        .map(|(_, value)| value.as_str())
        .collect();
    if states.len() != 1 || states[0] != expected_state {
        return Err("ChatGPT authentication state did not match. Try again.".to_string());
    }
    if params.iter().any(|(name, _)| name == "error") {
        return Err("ChatGPT authentication was rejected by the provider.".to_string());
    }
    let codes: Vec<_> = params
        .iter()
        .filter(|(name, _)| name == "code")
        .map(|(_, value)| value.as_str())
        .collect();
    if codes.len() != 1 || codes[0].is_empty() || codes[0].len() > 4096 {
        return Err("The ChatGPT callback did not contain a valid authorization code.".to_string());
    }
    Ok(Some(codes[0].to_string()))
}

pub async fn authenticate(config: &AssistantConfig) -> Result<Credentials, String> {
    let (listener, redirect_uri) = bind_callback_listener().await?;
    let (verifier, challenge) = generate_pkce();
    let state = random_urlsafe(32);
    let authorize_url = build_authorize_url(&challenge, &state, &redirect_uri);
    open::that(&authorize_url).map_err(|error| {
        format!(
            "Could not open the ChatGPT sign-in page: {}",
            redact(error.to_string())
        )
    })?;
    let (code, mut callback_stream) = wait_for_callback(listener, &state).await?;
    let result = async {
        let credentials = exchange_code(&code, &verifier, &redirect_uri, config).await?;
        save_credentials(&credentials)?;
        Ok(credentials)
    }
    .await;
    let _ = send_callback_page(&mut callback_stream, result.is_ok()).await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestVault {
        values: RefCell<HashMap<String, String>>,
        deleted: RefCell<Vec<String>>,
    }

    impl CredentialVault for TestVault {
        fn read(&self, user: &str) -> Result<Option<String>, String> {
            Ok(self.values.borrow().get(user).cloned())
        }

        fn delete(&self, user: &str) -> Result<(), String> {
            self.values.borrow_mut().remove(user);
            self.deleted.borrow_mut().push(user.to_string());
            Ok(())
        }
    }

    #[test]
    fn authorize_url_contains_pkce_and_callback() {
        let url = Url::parse(&build_authorize_url(
            "challenge",
            "state",
            "http://localhost:1455/auth/callback",
        ))
        .unwrap();
        let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(params.get("code_challenge").unwrap(), "challenge");
        assert_eq!(params.get("state").unwrap(), "state");
        assert_eq!(params.get("originator").unwrap(), "codex_cli_rs");
    }

    #[test]
    fn account_id_is_read_from_namespaced_claim() {
        let claims = serde_json::json!({
            "https://api.openai.com/auth": {"chatgpt_account_id": "acct_test"}
        });
        assert_eq!(
            account_id_from_claims(&claims).as_deref(),
            Some("acct_test")
        );
    }

    #[test]
    fn pkce_values_are_url_safe() {
        let (verifier, challenge) = generate_pkce();
        assert!(verifier.len() >= 43);
        assert_eq!(challenge.len(), 43);
        assert!(!verifier.contains('='));
        assert!(!challenge.contains('='));
    }

    #[test]
    fn oversized_credentials_are_chunked_below_windows_limit() {
        let credentials = Credentials {
            access_token: "a".repeat(6_000),
            refresh_token: "r".repeat(4_000),
            expires_at: 123,
            account_id: Some("account-test".to_string()),
        };
        let (manifest, chunks) = encode_credential_chunks(&credentials).unwrap();
        assert!(chunks.len() > 1);
        assert!(chunks
            .iter()
            .all(|chunk| chunk.encode_utf16().count() * 2 < 2560));
        let decoded = decode_credential_chunks(&manifest, &chunks).unwrap();
        assert_eq!(decoded.access_token, credentials.access_token);
        assert_eq!(decoded.refresh_token, credentials.refresh_token);
        assert_eq!(decoded.account_id, credentials.account_id);
    }

    #[test]
    fn corrupted_credential_chunk_is_rejected() {
        let credentials = Credentials {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at: 123,
            account_id: None,
        };
        let (manifest, mut chunks) = encode_credential_chunks(&credentials).unwrap();
        chunks[0].push('x');
        assert!(decode_credential_chunks(&manifest, &chunks).is_err());
    }

    #[test]
    fn credential_deletion_removes_chunks_before_manifest() {
        let vault = TestVault::default();
        let manifest = CredentialManifest {
            version: CREDENTIAL_FORMAT_VERSION,
            generation: "generation-test".to_string(),
            chunks: 2,
            checksum: "checksum".to_string(),
        };
        vault.values.borrow_mut().insert(
            CREDENTIAL_USER.to_string(),
            serde_json::to_string(&manifest).unwrap(),
        );
        for index in 0..manifest.chunks {
            vault.values.borrow_mut().insert(
                credential_chunk_user(&manifest.generation, index),
                "credential-data".to_string(),
            );
        }

        delete_credentials_from(&vault).unwrap();

        assert!(vault.values.borrow().is_empty());
        assert_eq!(
            vault.deleted.borrow().as_slice(),
            [
                credential_chunk_user(&manifest.generation, 0),
                credential_chunk_user(&manifest.generation, 1),
                CREDENTIAL_USER.to_string(),
            ]
        );
    }

    #[test]
    fn callback_requires_one_matching_state_and_one_code() {
        assert_eq!(
            callback_code("/auth/callback?state=expected&code=accepted", "expected").unwrap(),
            Some("accepted".to_string())
        );
        for target in [
            "/auth/callback?state=wrong&code=hidden-code",
            "/auth/callback?state=expected&state=expected&code=hidden-code",
            "/auth/callback?state=expected&code=first&code=second",
            "/auth/callback?state=expected",
        ] {
            let error = callback_code(target, "expected").unwrap_err();
            assert!(!error.contains("hidden-code"));
            assert!(!error.contains("first"));
            assert!(!error.contains("second"));
        }
    }

    #[test]
    fn callback_provider_error_is_generic_and_state_validated() {
        let error = callback_code(
            "/auth/callback?state=expected&error=denied-secret&error_description=body-secret",
            "expected",
        )
        .unwrap_err();
        assert_eq!(
            error,
            "ChatGPT authentication was rejected by the provider."
        );
        assert!(!error.contains("denied-secret"));
        assert!(!error.contains("body-secret"));
    }
}

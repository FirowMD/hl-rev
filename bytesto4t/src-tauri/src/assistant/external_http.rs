use super::network::validate_proxy_url;
use super::redaction::redact;
use crate::app_config::AssistantConfig;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const STATUS_MARKER: &str = "__BYTESTO4T_HTTP_STATUS__:";

#[derive(Debug)]
pub struct ExternalHttpResponse {
    pub status: u16,
    pub body: String,
}

pub async fn request(
    method: &str,
    url: &str,
    headers: Vec<(String, String)>,
    body: Option<String>,
    config: &AssistantConfig,
    timeout_seconds: u64,
) -> Result<ExternalHttpResponse, String> {
    let helper = configured_helper_path()?;
    let curl_config = build_curl_config(
        method,
        url,
        &headers,
        body.as_deref(),
        config,
        timeout_seconds,
    )?;
    tokio::task::spawn_blocking(move || run_curl(curl_config, helper))
        .await
        .map_err(|_| "External network helper task failed.".to_string())?
}

fn build_curl_config(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: Option<&str>,
    config: &AssistantConfig,
    timeout_seconds: u64,
) -> Result<String, String> {
    validate_proxy_url(&config.proxy_url)?;
    if headers
        .iter()
        .any(|(name, value)| name.contains(['\r', '\n']) || value.contains(['\r', '\n']))
    {
        return Err("External network helper rejected an invalid HTTP header.".to_string());
    }

    let mut output = String::from("silent\nshow-error\nno-buffer\n");
    push_option(&mut output, "request", method);
    push_option(&mut output, "url", url);
    push_option(&mut output, "connect-timeout", "30");
    push_option(&mut output, "max-time", &timeout_seconds.max(1).to_string());
    push_option(
        &mut output,
        "user-agent",
        &format!("bytesto4t/{}", env!("CARGO_PKG_VERSION")),
    );
    push_option(
        &mut output,
        "write-out",
        &format!("\n{STATUS_MARKER}%{{http_code}}"),
    );

    for (name, value) in headers {
        push_option(&mut output, "header", &format!("{name}: {value}"));
    }
    if let Some(body) = body {
        push_option(&mut output, "data-binary", body);
    }
    if config.disable_tls {
        output.push_str("insecure\n");
    }
    if config.bypass_proxy {
        push_option(&mut output, "noproxy", "*");
        push_option(&mut output, "proxy", "");
    } else if !config.proxy_url.trim().is_empty() {
        push_option(&mut output, "proxy", config.proxy_url.trim());
    }
    Ok(output)
}

fn push_option(output: &mut String, name: &str, value: &str) {
    output.push_str(name);
    output.push_str(" = \"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\r' => output.push_str("\\r"),
            '\n' => output.push_str("\\n"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
    output.push_str("\"\n");
}

pub(crate) fn helper_configured() -> bool {
    std::env::var("BYTESTO4T_HTTP_HELPER").is_ok_and(|value| !value.trim().is_empty())
}

fn validate_helper_path(configured: &str) -> Result<PathBuf, String> {
    let configured = configured.trim();
    let path = PathBuf::from(configured);
    if configured.is_empty() {
        return Err("BYTESTO4T_HTTP_HELPER is not configured.".to_string());
    }
    if !path.is_absolute() {
        return Err("BYTESTO4T_HTTP_HELPER must be an absolute executable path.".to_string());
    }
    if !path.is_file() {
        return Err("BYTESTO4T_HTTP_HELPER does not point to a file.".to_string());
    }
    path.canonicalize()
        .map_err(|_| "BYTESTO4T_HTTP_HELPER could not be resolved.".to_string())
}

fn configured_helper_path() -> Result<PathBuf, String> {
    let configured = std::env::var("BYTESTO4T_HTTP_HELPER").map_err(|_| {
        "External HTTP fallback is disabled. Set BYTESTO4T_HTTP_HELPER to the absolute path of a trusted curl-compatible executable to opt in.".to_string()
    })?;
    validate_helper_path(&configured)
}

fn run_curl(config: String, executable: PathBuf) -> Result<ExternalHttpResponse, String> {
    let helper = executable
        .file_name()
        .and_then(|name| name.to_str())
        .map(redact)
        .unwrap_or_else(|| "configured helper".to_string());
    let mut command = Command::new(&executable);
    command
        .args(["--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_window(&mut command);

    let mut child = command
        .spawn()
        .map_err(|_| format!("External network helper {helper} could not start."))?;
    let write_result = child
        .stdin
        .take()
        .ok_or_else(|| "External network helper stdin was unavailable.".to_string())?
        .write_all(config.as_bytes());
    if write_result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!(
            "External network helper {helper} could not receive the request via stdin."
        ));
    }

    let output = child
        .wait_with_output()
        .map_err(|_| "External network helper failed while waiting for completion.".to_string())?;
    if !output.status.success() {
        return Err(format!(
            "External network helper {helper} exited with status {}.",
            output.status
        ));
    }
    parse_output(&output.stdout)
}

fn parse_output(output: &[u8]) -> Result<ExternalHttpResponse, String> {
    let output = String::from_utf8_lossy(output);
    let marker_index = output
        .rfind(STATUS_MARKER)
        .ok_or_else(|| "External network helper returned no HTTP status.".to_string())?;
    let status = output[marker_index + STATUS_MARKER.len()..]
        .trim()
        .parse::<u16>()
        .map_err(|error| format!("External network helper returned an invalid status: {error}"))?;
    let body = output[..marker_index]
        .strip_suffix('\n')
        .unwrap_or(&output[..marker_index])
        .to_string();
    Ok(ExternalHttpResponse { status, body })
}

#[cfg(windows)]
fn hide_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
fn hide_window(_command: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn curl_config_keeps_secrets_in_stdin_payload() {
        let config = build_curl_config(
            "POST",
            "https://example.test/responses",
            &[("Authorization".to_string(), "Bearer secret".to_string())],
            Some(r#"{"message":"quoted \\\"value\\\""}"#),
            &AssistantConfig::default(),
            60,
        )
        .unwrap();
        assert!(config.contains("Authorization: Bearer secret"));
        assert!(config.contains("data-binary"));
        assert!(!config.lines().next().unwrap().contains("secret"));
    }

    #[test]
    fn helper_requires_an_explicit_absolute_file_path() {
        let error = validate_helper_path("curl").unwrap_err();
        assert!(error.contains("absolute"));
    }

    #[test]
    fn curl_config_rejects_credential_bearing_proxy() {
        let settings = AssistantConfig {
            proxy_url: "http://user:password@proxy.example:8080".to_string(),
            ..AssistantConfig::default()
        };
        let error =
            build_curl_config("GET", "https://example.test", &[], None, &settings, 5).unwrap_err();
        assert!(!error.contains("user"));
        assert!(!error.contains("password"));
    }

    #[test]
    fn helper_output_separates_body_and_status() {
        let response = parse_output(
            b"data: {\"type\":\"response.completed\"}\n\n__BYTESTO4T_HTTP_STATUS__:200",
        )
        .unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.starts_with("data:"));
    }

    #[cfg(windows)]
    #[test]
    fn curl_helper_round_trips_through_local_http_server() {
        let where_output = Command::new("where.exe").arg("curl.exe").output().unwrap();
        if !where_output.status.success() {
            return;
        }
        let executable = String::from_utf8_lossy(&where_output.stdout)
            .lines()
            .next()
            .map(PathBuf::from)
            .unwrap();

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut request = [0_u8; 4096];
            let count = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..count]);
            assert!(request.contains("Authorization: Bearer test-token"));
            assert!(request.contains(r#"{"probe":true}"#));
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .unwrap();
        });

        let settings = AssistantConfig {
            bypass_proxy: true,
            ..AssistantConfig::default()
        };
        let config = build_curl_config(
            "POST",
            &format!("http://{address}/probe"),
            &[("Authorization".to_string(), "Bearer test-token".to_string())],
            Some(r#"{"probe":true}"#),
            &settings,
            5,
        )
        .unwrap();
        let response = run_curl(config, executable).unwrap();
        server.join().unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "ok");
    }
}

use super::network::validate_proxy_url;
use super::redaction::redact;
use crate::app_config::AssistantConfig;
use std::io::Write;
use std::path::Path;
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
    let curl_config = build_curl_config(
        method,
        url,
        &headers,
        body.as_deref(),
        config,
        timeout_seconds,
    )?;
    tokio::task::spawn_blocking(move || run_curl(curl_config))
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

fn helper_commands() -> Vec<String> {
    let mut commands = Vec::new();
    if let Ok(configured) = std::env::var("BYTESTO4T_HTTP_HELPER") {
        let configured = configured.trim();
        if !configured.is_empty() {
            commands.push(configured.to_string());
        }
    }
    if cfg!(windows) {
        commands.push("curl.exe".to_string());
    } else {
        commands.push("curl".to_string());
    }
    commands.dedup();
    commands
}

fn run_curl(config: String) -> Result<ExternalHttpResponse, String> {
    let mut failures = Vec::new();
    for executable in helper_commands() {
        let helper = Path::new(&executable)
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

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => {
                failures.push(format!("{helper}: could not start"));
                continue;
            }
        };
        let write_result = child
            .stdin
            .take()
            .ok_or_else(|| "External network helper stdin was unavailable.".to_string())?
            .write_all(config.as_bytes());
        if write_result.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            failures.push(format!("{helper}: could not receive the request via stdin"));
            continue;
        }

        let output = child.wait_with_output().map_err(|_| {
            "External network helper failed while waiting for completion.".to_string()
        })?;
        if !output.status.success() {
            failures.push(format!("{helper}: exited with status {}", output.status));
            continue;
        }
        return parse_output(&output.stdout);
    }

    Err(format!(
        "No external curl transport succeeded. Set BYTESTO4T_HTTP_HELPER to a curl-compatible executable. {}",
        failures.join("; ")
    ))
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
        assert!(!helper_commands()
            .iter()
            .any(|value| value.contains("secret")));
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
        if Command::new("curl.exe").arg("--version").output().is_err() {
            return;
        }

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
        let response = run_curl(config).unwrap();
        server.join().unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "ok");
    }
}

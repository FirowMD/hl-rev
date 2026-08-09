use crate::app_config::AssistantConfig;
use reqwest::Client;
use std::error::Error;
use std::time::Duration;
use url::Url;

pub fn validate_proxy_url(value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(());
    }
    let url = Url::parse(value).map_err(|_| "Invalid assistant proxy URL.".to_string())?;
    if !matches!(url.scheme(), "http" | "https" | "socks5") {
        return Err("Proxy URL must use http, https, or socks5.".to_string());
    }
    if url.host_str().is_none() {
        return Err("Proxy URL must include a host.".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(
            "Proxy credentials cannot be embedded in the URL; credential-vault proxy authentication is not configured."
                .to_string(),
        );
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("Proxy URL cannot contain a query string or fragment.".to_string());
    }
    Ok(())
}

pub fn build_client(config: &AssistantConfig, timeout_seconds: u64) -> Result<Client, String> {
    validate_proxy_url(&config.proxy_url)?;
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .connect_timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(config.disable_tls)
        .user_agent(format!("bytesto4t/{}", env!("CARGO_PKG_VERSION")));

    if config.bypass_proxy {
        builder = builder.no_proxy();
    } else if !config.proxy_url.trim().is_empty() {
        let proxy = reqwest::Proxy::all(config.proxy_url.trim())
            .map_err(|_| "Invalid assistant proxy URL.".to_string())?;
        builder = builder.proxy(proxy);
    }

    builder.build().map_err(|error| {
        format!(
            "Could not configure the assistant network client: {}. Check the VPN/proxy and TLS settings.",
            super::redaction::redact(error.to_string())
        )
    })
}

pub fn network_error(context: &str, error: impl std::fmt::Display) -> String {
    format!(
        "{context}: {}. The assistant uses the system proxy and VPN route by default. Check the custom proxy setting, or enable proxy bypass only when the VPN provides a direct route.",
        super::redaction::redact(error.to_string())
    )
}

pub fn is_access_denied(error: &(dyn Error + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(source) = current {
        let message = source.to_string().to_ascii_lowercase();
        if message.contains("access is denied")
            || message.contains("permission denied")
            || message.contains("os error 10013")
        {
            return true;
        }
        if source
            .downcast_ref::<std::io::Error>()
            .is_some_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied)
        {
            return true;
        }
        current = source.source();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_custom_proxy_is_rejected() {
        let config = AssistantConfig {
            proxy_url: "not a proxy URL".to_string(),
            ..AssistantConfig::default()
        };
        assert!(build_client(&config, 1).is_err());
    }

    #[test]
    fn supported_proxy_schemes_are_accepted() {
        for proxy in [
            "http://proxy.example:8080",
            "https://proxy.example:8443",
            "socks5://127.0.0.1:1080",
        ] {
            assert!(validate_proxy_url(proxy).is_ok(), "{proxy}");
        }
    }

    #[test]
    fn proxy_credentials_query_and_fragment_are_rejected() {
        for proxy in [
            "http://user@proxy.example:8080",
            "https://user:password@proxy.example:8443",
            "socks5://proxy.example:1080?token=value",
            "http://proxy.example:8080#credentials",
        ] {
            let error = validate_proxy_url(proxy).unwrap_err();
            assert!(!error.contains("password"));
            assert!(!error.contains("value"));
        }
    }

    #[test]
    fn bypass_does_not_skip_proxy_validation() {
        let config = AssistantConfig {
            proxy_url: "https://user:password@proxy.example".to_string(),
            bypass_proxy: true,
            ..AssistantConfig::default()
        };
        assert!(build_client(&config, 1).is_err());
    }

    #[test]
    fn permission_denied_is_detected_through_error_chain() {
        let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "socket blocked");
        assert!(is_access_denied(&error));
    }
}

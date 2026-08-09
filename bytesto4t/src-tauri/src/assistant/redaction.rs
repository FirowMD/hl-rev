use regex::{Captures, Regex};
use std::sync::LazyLock;
use url::Url;

const REDACTED: &str = "[REDACTED]";

static URL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\b(?:https?|socks5)://[^\s<>\"']+"#).expect("URL redaction regex is valid")
});
static PARAM_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b(code|authorization_code|state|token|access_token|refresh_token|id_token|api_key|client_secret|code_verifier|account_id|chatgpt_account_id|password|proxy_password)=([^&\s\"']*)"#,
    )
    .expect("parameter redaction regex is valid")
});
static JSON_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(\"(?:code|authorization_code|state|token|access_token|refresh_token|id_token|api_key|client_secret|code_verifier|account_id|chatgpt_account_id|password|proxy_password)\"\s*:\s*)\"(?:\\.|[^\"])*\""#,
    )
    .expect("JSON redaction regex is valid")
});
static HEADER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?im)\b(authorization|proxy-authorization|chatgpt-account-id|x-api-key)\s*:\s*[^\r\n;]+",
    )
    .expect("header redaction regex is valid")
});
static BEARER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bbearer\s+[A-Za-z0-9._~+/=-]+")
        .expect("bearer-token redaction regex is valid")
});
static JWT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\beyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b")
        .expect("JWT redaction regex is valid")
});

fn sensitive_parameter(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "code"
            | "authorization_code"
            | "state"
            | "token"
            | "access_token"
            | "refresh_token"
            | "id_token"
            | "api_key"
            | "client_secret"
            | "code_verifier"
            | "account_id"
            | "chatgpt_account_id"
            | "password"
            | "proxy_password"
    )
}

fn redact_url(value: &str) -> String {
    let Ok(mut url) = Url::parse(value) else {
        return value.to_string();
    };
    if !url.username().is_empty() || url.password().is_some() {
        let _ = url.set_username("redacted");
        let _ = url.set_password(None);
    }
    if url.query().is_some() {
        let pairs: Vec<_> = url.query_pairs().into_owned().collect();
        url.query_pairs_mut()
            .clear()
            .extend_pairs(pairs.iter().map(|(name, value)| {
                (
                    name.as_str(),
                    if sensitive_parameter(name) {
                        REDACTED
                    } else {
                        value
                    },
                )
            }));
    }
    if let Some(fragment) = url.fragment().map(str::to_string) {
        let redacted = PARAM_PATTERN.replace_all(&fragment, |captures: &Captures<'_>| {
            format!("{}={REDACTED}", &captures[1])
        });
        url.set_fragment(Some(&redacted));
    }
    url.to_string()
}

pub fn redact(value: impl AsRef<str>) -> String {
    let value = URL_PATTERN.replace_all(value.as_ref(), |captures: &Captures<'_>| {
        redact_url(&captures[0])
    });
    let value = PARAM_PATTERN.replace_all(&value, |captures: &Captures<'_>| {
        format!("{}={REDACTED}", &captures[1])
    });
    let value = JSON_PATTERN.replace_all(&value, |captures: &Captures<'_>| {
        format!(r#"{}\"{REDACTED}\""#, &captures[1])
    });
    let value = HEADER_PATTERN.replace_all(&value, |captures: &Captures<'_>| {
        format!("{}: {REDACTED}", &captures[1])
    });
    let value = BEARER_PATTERN.replace_all(&value, format!("Bearer {REDACTED}"));
    JWT_PATTERN.replace_all(&value, REDACTED).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_url_userinfo_and_sensitive_query_parameters() {
        let output = redact(
            "failed https://alice:proxy-password@example.test/cb?code=oauth-code&safe=yes&state=csrf#token=fragment-token",
        );
        assert!(!output.contains("alice"));
        assert!(!output.contains("proxy-password"));
        assert!(!output.contains("oauth-code"));
        assert!(!output.contains("csrf"));
        assert!(!output.contains("fragment-token"));
        assert!(output.contains("safe=yes"));
    }

    #[test]
    fn redacts_forms_json_and_authorization_headers() {
        let output = redact(
            "refresh_token=form-secret&code_verifier=pkce-secret\n\"access_token\":\"json-secret\"\nAuthorization: Bearer header-secret\nChatGPT-Account-ID: account-secret",
        );
        for secret in [
            "form-secret",
            "pkce-secret",
            "json-secret",
            "header-secret",
            "account-secret",
        ] {
            assert!(!output.contains(secret));
        }
    }

    #[test]
    fn redacts_every_sensitive_parameter_and_token_shape() {
        let names = [
            "code",
            "state",
            "token",
            "access_token",
            "refresh_token",
            "id_token",
            "api_key",
            "client_secret",
            "code_verifier",
            "account_id",
            "password",
        ];
        for name in names {
            let secret = format!("unique-{name}-value");
            let output = redact(format!("{name}={secret}"));
            assert!(!output.contains(&secret), "failed to redact {name}");
        }
        let output =
            redact("Bearer bearer-secret eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyIn0.signature");
        assert!(!output.contains("bearer-secret"));
        assert!(!output.contains("eyJhbGci"));
    }
}

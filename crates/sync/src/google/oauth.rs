use crate::error::SyncError;
use crate::secrets::SecretStore;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_CONTACTS_SCOPE: &str = "https://www.googleapis.com/auth/contacts";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTokenBundle {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub fn google_secret_key(profile_id: profile_pulse_core::ProfileId) -> String {
    format!("google:{}", profile_id.0)
}

pub fn load_google_tokens(
    secrets: &SecretStore,
    profile_id: profile_pulse_core::ProfileId,
) -> Result<Option<GoogleTokenBundle>, SyncError> {
    let Some(raw) = secrets.get(&google_secret_key(profile_id))? else {
        return Ok(None);
    };
    serde_json::from_str(&raw)
        .map_err(|e| SyncError::OAuth(e.to_string()))
        .map(Some)
}

pub fn store_google_tokens(
    secrets: &SecretStore,
    profile_id: profile_pulse_core::ProfileId,
    tokens: &GoogleTokenBundle,
) -> Result<(), SyncError> {
    let raw = serde_json::to_string(tokens).map_err(|e| SyncError::OAuth(e.to_string()))?;
    secrets.put(&google_secret_key(profile_id), &raw)
}

pub async fn authorize_google_pkce(
    client_id: &str,
    secrets: &SecretStore,
    profile_id: profile_pulse_core::ProfileId,
) -> Result<GoogleTokenBundle, SyncError> {
    if client_id.trim().is_empty() {
        return Err(SyncError::NotConfigured(
            "set PROFILE_PULSE_GOOGLE_CLIENT_ID to link Google Contacts".into(),
        ));
    }
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| SyncError::OAuth(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| SyncError::OAuth(e.to_string()))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}/oauth/callback");
    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_auth_uri(
            AuthUrl::new(GOOGLE_AUTH_URL.to_string())
                .map_err(|e| SyncError::OAuth(e.to_string()))?,
        )
        .set_token_uri(
            TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                .map_err(|e| SyncError::OAuth(e.to_string()))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_uri.clone()).map_err(|e| SyncError::OAuth(e.to_string()))?,
        );
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, _csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(GOOGLE_CONTACTS_SCOPE.to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();
    open_browser(auth_url.as_ref())?;
    let http_client = Client::new();
    let code = receive_oauth_code(listener).await?;
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|e| SyncError::OAuth(e.to_string()))?;
    let bundle = GoogleTokenBundle {
        access_token: token.access_token().secret().clone(),
        refresh_token: token.refresh_token().map(|t| t.secret().clone()),
        expires_at: token.expires_in().map(|duration| {
            chrono::Utc::now() + chrono::Duration::from_std(duration).unwrap_or_default()
        }),
    };
    store_google_tokens(secrets, profile_id, &bundle)?;
    Ok(bundle)
}

async fn receive_oauth_code(listener: TcpListener) -> Result<String, SyncError> {
    let (mut stream, _) = tokio::time::timeout(Duration::from_secs(300), listener.accept())
        .await
        .map_err(|_| SyncError::OAuth("timed out waiting for Google sign-in".into()))?
        .map_err(|e| SyncError::OAuth(e.to_string()))?;
    let mut buffer = vec![0u8; 4096];
    let read = stream
        .read(&mut buffer)
        .await
        .map_err(|e| SyncError::OAuth(e.to_string()))?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    let code = url::Url::parse(&format!("http://localhost{path}"))
        .ok()
        .and_then(|url| {
            url.query_pairs()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| v.into_owned())
        })
        .ok_or_else(|| SyncError::OAuth("missing authorization code in callback".into()))?;
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
        <html><body><p>Signed in. You can close this window and return to Profile Pulse.</p></body></html>";
    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|e| SyncError::OAuth(e.to_string()))?;
    Ok(code)
}

fn open_browser(url: &str) -> Result<(), SyncError> {
    #[cfg(target_os = "linux")]
    let status = std::process::Command::new("xdg-open").arg(url).status();
    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open").arg(url).status();
    #[cfg(windows)]
    let status = std::process::Command::new("cmd")
        .args(["/C", "start", url])
        .status();
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    let status: Result<std::process::ExitStatus, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "browser",
    ));
    status.map_err(|e| SyncError::OAuth(format!("failed to open browser: {e}")))?;
    Ok(())
}

pub async fn refresh_google_access_token(
    client_id: &str,
    secrets: &SecretStore,
    profile_id: profile_pulse_core::ProfileId,
) -> Result<String, SyncError> {
    let mut bundle = load_google_tokens(secrets, profile_id)?
        .ok_or_else(|| SyncError::AuthRequired("Google Contacts".into()))?;
    if let Some(expires_at) = bundle.expires_at
        && expires_at > chrono::Utc::now() + chrono::Duration::minutes(2)
    {
        return Ok(bundle.access_token);
    }
    let Some(refresh_token) = bundle.refresh_token.clone() else {
        return Ok(bundle.access_token);
    };
    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_auth_uri(
            AuthUrl::new(GOOGLE_AUTH_URL.to_string())
                .map_err(|e| SyncError::OAuth(e.to_string()))?,
        )
        .set_token_uri(
            TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                .map_err(|e| SyncError::OAuth(e.to_string()))?,
        );
    let http_client = Client::new();
    let token = client
        .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token))
        .request_async(&http_client)
        .await
        .map_err(|e| SyncError::OAuth(e.to_string()))?;
    bundle.access_token = token.access_token().secret().clone();
    if let Some(new_refresh) = token.refresh_token() {
        bundle.refresh_token = Some(new_refresh.secret().clone());
    }
    bundle.expires_at = token.expires_in().map(|duration| {
        chrono::Utc::now() + chrono::Duration::from_std(duration).unwrap_or_default()
    });
    store_google_tokens(secrets, profile_id, &bundle)?;
    Ok(bundle.access_token)
}

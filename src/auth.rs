use librespot_oauth::{OAuthClientBuilder, OAuthToken};
use rspotify::oauth2::TokenInfo;
use std::net::TcpListener;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Refresh an access token using PKCE (no client_secret).
/// rspotify's built-in refresh uses client_credentials which Spotify
/// now rejects with "Only code_challenge_method S256 is supported".
/// librespot-oauth's refresh_token() is a direct HTTP call, no browser.
pub fn pkce_refresh_token(client_id: &str, redirect_uri: &str, refresh_token: &str) -> Option<TokenInfo> {
  let builder = OAuthClientBuilder::new(client_id, redirect_uri, SCOPES.to_vec());
  let client = builder.build().ok()?;
  let token = client.refresh_token(refresh_token).ok()?;
  Some(map_token(token))
}

pub const NCSPOT_CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";
pub const NCSPOT_REDIRECT_URI: &str = "http://127.0.0.1:8989/login";

pub static SCOPES: &[&str] = &[
  "playlist-read-collaborative",
  "playlist-read-private",
  "playlist-modify-private",
  "playlist-modify-public",
  "user-follow-read",
  "user-follow-modify",
  "user-library-modify",
  "user-library-read",
  "user-modify-playback-state",
  "user-read-currently-playing",
  "user-read-playback-state",
  "user-read-playback-position",
  "user-read-private",
  "user-read-recently-played",
];

pub fn find_free_port() -> u16 {
  TcpListener::bind("127.0.0.1:0")
    .and_then(|socket| socket.local_addr())
    .map(|addr| addr.port())
    .unwrap_or(8888)
}

pub fn get_redirect_uri(port: u16) -> String {
  format!("http://127.0.0.1:{}/login", port)
}

pub fn map_token(token: OAuthToken) -> TokenInfo {
  let now = Instant::now();
  let duration = if token.expires_at > now {
    token.expires_at.duration_since(now)
  } else {
    std::time::Duration::from_secs(0)
  };

  let expires_at = SystemTime::now()
    .checked_add(duration)
    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
    .map(|d| d.as_secs() as i64);

  TokenInfo {
    access_token: token.access_token,
    token_type: "Bearer".to_string(),
    expires_in: duration.as_secs() as u32,
    refresh_token: Some(token.refresh_token),
    scope: SCOPES.join(" "),
    expires_at,
  }
}

pub fn get_new_token(client_id: &str, redirect_uri: &str) -> Option<TokenInfo> {
  let builder = OAuthClientBuilder::new(client_id, redirect_uri, SCOPES.to_vec());

  match builder.build() {
    Ok(client) => {
      println!("Opening browser for Spotify authentication...");
      client.get_access_token().ok().map(map_token)
    }
    Err(_) => None,
  }
}

pub fn refresh_token(
  client_id: &str,
  redirect_uri: &str,
  refresh_token: &str,
) -> Option<TokenInfo> {
  let builder = OAuthClientBuilder::new(client_id, redirect_uri, SCOPES.to_vec());

  match builder.build() {
    Ok(client) => client.refresh_token(refresh_token).ok().map(map_token),
    Err(_) => None,
  }
}

/// Try to bootstrap a spt token from spotify-player's cached token for the same client_id.
/// This avoids a browser re-auth when spotify-player has already authenticated the same app.
pub fn bootstrap_from_spotify_player_cache(client_id: &str) -> Option<TokenInfo> {
  let cache_path = dirs::cache_dir()?
    .join("spotify-player")
    .join(format!("{}_token.json", client_id));

  let content = std::fs::read_to_string(&cache_path).ok()?;
  let v: serde_json::Value = serde_json::from_str(&content).ok()?;
  let refresh_token = v["refresh_token"].as_str()?.to_string();

  // Use a plain HTTP refresh — no redirect_uri flow needed for refresh grants.
  // The refresh_token is valid regardless of which redirect_uri was used at auth time.
  let output = std::process::Command::new("curl")
    .args([
      "--silent", "--max-time", "10",
      "-X", "POST", "https://accounts.spotify.com/api/token",
      "-d", &format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}",
        refresh_token, client_id
      ),
      "-H", "Content-Type: application/x-www-form-urlencoded",
    ])
    .output()
    .ok()?;

  if !output.status.success() {
    return None;
  }

  let tok: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
  let access_token = tok["access_token"].as_str()?.to_string();
  let expires_in = tok["expires_in"].as_u64().unwrap_or(3600) as u32;
  let new_refresh = tok["refresh_token"]
    .as_str()
    .unwrap_or(&refresh_token)
    .to_string();

  let now = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .ok()?
    .as_secs() as i64;

  Some(TokenInfo {
    access_token,
    token_type: "Bearer".to_string(),
    expires_in,
    refresh_token: Some(new_refresh),
    scope: SCOPES.join(" "),
    expires_at: Some(now + expires_in as i64),
  })
}

use librespot_oauth::{OAuthClientBuilder, OAuthToken};
use rspotify::oauth2::TokenInfo;
use std::net::TcpListener;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub const NCSPOT_CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";

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

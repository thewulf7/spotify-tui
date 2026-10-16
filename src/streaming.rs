/// Integrated Spotify Connect streaming device using librespot.
///
/// When enabled via `streaming::start()`, spt registers itself as a
/// Spotify Connect "Computer" device using a dedicated librespot OAuth token
/// (Spotify's official client ID — the only one that works with librespot's AP).
/// Playback commands route audio through the local audio stack (rodio → CoreAudio).
///
/// Runs on a dedicated tokio-1 single-thread runtime so it does not conflict
/// with the legacy tokio-0.2 runtime used by the rest of spt.
use anyhow::{Context, Result};
use librespot_connect::{ConnectConfig, Spirc};
use librespot_core::{
  authentication::Credentials,
  cache::Cache,
  config::{DeviceType, SessionConfig},
  Session,
};
use librespot_oauth::OAuthClientBuilder;
use librespot_playback::{
  audio_backend,
  config::{AudioFormat, Bitrate, PlayerConfig},
  mixer::{self, softmixer::SoftMixer, Mixer, MixerConfig},
  player::Player,
};
use std::{fs, path::PathBuf, sync::Arc};

/// Spotify's official client ID — the one librespot was built against.
/// PKCE/Web-API tokens from other client IDs are rejected by the AP with INVALID_CREDENTIALS.
pub const SPOTIFY_OFFICIAL_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";

/// Scopes for the librespot streaming client.
pub const STREAMING_SCOPES: &[&str] = &[
  "user-read-playback-state",
  "user-modify-playback-state",
  "user-read-currently-playing",
  "app-remote-control",
  "streaming",
  "playlist-read-private",
  "playlist-read-collaborative",
  "playlist-modify-private",
  "playlist-modify-public",
  "user-follow-modify",
  "user-follow-read",
  "user-read-playback-position",
  "user-top-read",
  "user-read-recently-played",
  "user-library-modify",
  "user-library-read",
  "user-personalized",
];

/// Cached streaming token file in our config/cache dir.
const STREAMING_TOKEN_FILE: &str = ".spt_streaming_token.json";

pub struct StreamingHandle {
  spirc: Spirc,
  _rt: tokio1::runtime::Runtime,
}

impl StreamingHandle {
  pub fn stop(self) {
    let _ = self.spirc.shutdown();
  }
}

pub struct StreamingConfig {
  pub cache_dir: PathBuf,
  pub device_name: String,
  pub volume: u8,
  pub bitrate: u32,
}

fn load_streaming_refresh_token(cache_dir: &PathBuf) -> Option<String> {
  let path = cache_dir.join(STREAMING_TOKEN_FILE);
  let s = fs::read_to_string(&path).ok()?;
  let v: serde_json::Value = serde_json::from_str(&s).ok()?;
  v["refresh_token"].as_str().map(|s| s.to_string())
}

fn save_streaming_tokens(cache_dir: &PathBuf, refresh_token: &str, access_token: &str) {
  let path = cache_dir.join(STREAMING_TOKEN_FILE);
  let v = serde_json::json!({
      "refresh_token": refresh_token,
      "access_token": access_token,
  });
  let _ = fs::create_dir_all(cache_dir);
  let _ = fs::write(&path, serde_json::to_string(&v).unwrap_or_default());
}

/// Obtain a valid librespot access token:
/// 1. Try refresh_token from cache (silent, no browser).
/// 2. Fall back to full PKCE flow (opens browser once, same redirect port logic as spt auth).
fn obtain_streaming_token(cache_dir: &PathBuf, redirect_uri: &str) -> Result<String> {
  let builder = OAuthClientBuilder::new(
    SPOTIFY_OFFICIAL_CLIENT_ID,
    redirect_uri,
    STREAMING_SCOPES.to_vec(),
  );
  let client = builder.build().map_err(|e| anyhow::anyhow!("{e}"))?;

  if let Some(refresh) = load_streaming_refresh_token(cache_dir) {
    match client.refresh_token(&refresh) {
      Ok(token) => {
        save_streaming_tokens(cache_dir, &token.refresh_token, &token.access_token);
        return Ok(token.access_token);
      }
      Err(e) => {
        eprintln!("Streaming token refresh failed ({e}), re-authenticating...");
      }
    }
  }

  eprintln!("Authenticating spt streaming device (opens browser once)...");
  let token = client
    .get_access_token()
    .map_err(|e| anyhow::anyhow!("{e}"))?;
  save_streaming_tokens(cache_dir, &token.refresh_token, &token.access_token);
  Ok(token.access_token)
}

pub fn start(config: StreamingConfig) -> Result<StreamingHandle> {
  let redirect_uri = {
    let port = crate::auth::find_free_port();
    crate::auth::get_redirect_uri(port)
  };

  let access_token = obtain_streaming_token(&config.cache_dir, &redirect_uri)?;

  // Use a multi-thread runtime so the Spirc task keeps running
  // after block_on returns. A current_thread runtime drops spawned
  // tasks as soon as block_on() completes.
  let rt = tokio1::runtime::Builder::new_multi_thread()
    .worker_threads(2)
    .enable_all()
    .build()
    .context("build librespot tokio runtime")?;

  let spirc = rt.block_on(async { connect_device(&config, &access_token).await })?;

  Ok(StreamingHandle { spirc, _rt: rt })
}

async fn connect_device(config: &StreamingConfig, access_token: &str) -> Result<Spirc> {
  let cache =
    Cache::new(Some(config.cache_dir.clone()), None, None, None).context("open librespot cache")?;

  let creds = cache
    .credentials()
    .unwrap_or_else(|| Credentials::with_access_token(access_token));

  let session_config = SessionConfig::default();
  // NOTE: do NOT call session.connect() — Spirc::new() connects the session internally.
  let session = Session::new(session_config, Some(cache));

  let vol_u16 = (f64::from(config.volume.min(100)) / 100.0 * 65535.0).round() as u16;

  let mixer = Arc::new(SoftMixer::open(MixerConfig::default()).context("open softmixer")?);
  mixer.set_volume(vol_u16);

  let bitrate = match config.bitrate {
    96 => Bitrate::Bitrate96,
    160 => Bitrate::Bitrate160,
    _ => Bitrate::Bitrate320,
  };

  let player_config = PlayerConfig {
    bitrate,
    ..Default::default()
  };

  let backend = audio_backend::find(None).expect("no audio backend compiled in");
  let player = Player::new(
    player_config,
    session.clone(),
    mixer.get_soft_volume(),
    move || backend(None, AudioFormat::default()),
  );

  let connect_config = ConnectConfig {
    name: config.device_name.clone(),
    device_type: DeviceType::Computer,
    initial_volume: vol_u16,
    is_group: false,
    disable_volume: false,
    volume_steps: 64,
  };

  let (spirc, spirc_task) = Spirc::new(connect_config, session, creds, player, mixer)
    .await
    .context("initialise Spirc")?;

  tokio1::spawn(spirc_task);

  Ok(spirc)
}

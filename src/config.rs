use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
};

const DEFAULT_PORT: u16 = 8888;
const FILE_NAME: &str = "client.yml";
const CONFIG_DIR: &str = ".config";
const APP_CONFIG_DIR: &str = "spotify-tui";
const TOKEN_CACHE_FILE: &str = ".spotify_token_cache.json";

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientConfig {
  pub client_id: String,
  pub client_secret: String,
  pub device_id: Option<String>,
  // FIXME: port should be defined in `user_config` not in here
  pub port: Option<u16>,
  /// Override redirect URI. If set, used verbatim (enables stdin PKCE flow
  /// when no port is present, e.g. "http://127.0.0.1/callback").
  pub redirect_uri: Option<String>,
}

pub struct ConfigPaths {
  pub config_file_path: PathBuf,
  pub token_cache_path: PathBuf,
}

impl ClientConfig {
  pub fn new() -> ClientConfig {
    ClientConfig {
      client_id: "".to_string(),
      client_secret: "".to_string(),
      device_id: None,
      port: None,
      redirect_uri: None,
    }
  }

  pub fn get_redirect_uri(&self) -> String {
    if let Some(ref uri) = self.redirect_uri {
      return uri.clone();
    }
    format!("http://127.0.0.1:{}/callback", self.get_port())
  }

  pub fn get_port(&self) -> u16 {
    self.port.unwrap_or(DEFAULT_PORT)
  }

  pub fn get_or_build_paths(&self) -> Result<ConfigPaths> {
    match dirs::home_dir() {
      Some(home) => {
        let path = Path::new(&home);
        let home_config_dir = path.join(CONFIG_DIR);
        let app_config_dir = home_config_dir.join(APP_CONFIG_DIR);

        if !home_config_dir.exists() {
          fs::create_dir(&home_config_dir)?;
        }

        if !app_config_dir.exists() {
          fs::create_dir(&app_config_dir)?;
        }

        let config_file_path = &app_config_dir.join(FILE_NAME);
        let token_cache_path = &app_config_dir.join(TOKEN_CACHE_FILE);

        let paths = ConfigPaths {
          config_file_path: config_file_path.to_path_buf(),
          token_cache_path: token_cache_path.to_path_buf(),
        };

        Ok(paths)
      }
      None => Err(anyhow!("No $HOME directory found for client config")),
    }
  }

  pub fn set_device_id(&mut self, device_id: String) -> Result<()> {
    let paths = self.get_or_build_paths()?;
    let config_string = fs::read_to_string(&paths.config_file_path)?;
    let mut config_yml: ClientConfig = serde_yaml::from_str(&config_string)?;

    self.device_id = Some(device_id.clone());
    config_yml.device_id = Some(device_id);

    let new_config = serde_yaml::to_string(&config_yml)?;
    let mut config_file = fs::File::create(&paths.config_file_path)?;
    write!(config_file, "{}", new_config)?;
    Ok(())
  }

  pub fn load_config(&mut self) -> Result<()> {
    let paths = self.get_or_build_paths()?;
    if paths.config_file_path.exists() {
      let config_string = fs::read_to_string(&paths.config_file_path)?;
      let config_yml: ClientConfig = serde_yaml::from_str(&config_string)?;

      self.client_id = config_yml.client_id;
      self.client_secret = config_yml.client_secret;
      self.device_id = config_yml.device_id;
      self.port = config_yml.port;

      Ok(())
    } else {
      let config_yml = ClientConfig {
        client_id: "".to_string(),
        client_secret: "".to_string(),
        device_id: None,
        port: Some(DEFAULT_PORT),
        redirect_uri: None,
      };

      let content_yml = serde_yaml::to_string(&config_yml)?;

      let mut new_config = fs::File::create(&paths.config_file_path)?;
      write!(new_config, "{}", content_yml)?;

      self.port = Some(DEFAULT_PORT);

      Ok(())
    }
  }
}

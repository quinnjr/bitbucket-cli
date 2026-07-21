use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "bitbucket-cli";
const CONFIG_FILE: &str = "config.toml";

/// XDG Base Directory helper functions
///
/// On Linux, these follow the XDG Base Directory Specification:
/// - Config: `$XDG_CONFIG_HOME/bitbucket-cli` (default: `~/.config/bitbucket-cli`)
/// - Data: `$XDG_DATA_HOME/bitbucket-cli` (default: `~/.local/share/bitbucket-cli`)
/// - Cache: `$XDG_CACHE_HOME/bitbucket-cli` (default: `~/.cache/bitbucket-cli`)
/// - State: `$XDG_STATE_HOME/bitbucket-cli` (default: `~/.local/state/bitbucket-cli`)
///
/// On macOS:
/// - Config: `~/Library/Application Support/bitbucket-cli`
/// - Data: `~/Library/Application Support/bitbucket-cli`
/// - Cache: `~/Library/Caches/bitbucket-cli`
///
/// On Windows:
/// - Config: `%APPDATA%\bitbucket-cli`
/// - Data: `%APPDATA%\bitbucket-cli`
/// - Cache: `%LOCALAPPDATA%\bitbucket-cli`
pub mod xdg {
    use super::*;

    /// Get the XDG config directory for the application
    ///
    /// Respects `$XDG_CONFIG_HOME` on Linux (falls back to `~/.config`)
    pub fn config_dir() -> Result<PathBuf> {
        // First check for explicit XDG_CONFIG_HOME on Unix
        #[cfg(unix)]
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg_config.is_empty() {
                return Ok(PathBuf::from(xdg_config).join(APP_NAME));
            }
        }

        dirs::config_dir()
            .map(|p| p.join(APP_NAME))
            .context("Could not determine config directory")
    }

    /// Get the XDG data directory for the application
    ///
    /// Respects `$XDG_DATA_HOME` on Linux (falls back to `~/.local/share`)
    pub fn data_dir() -> Result<PathBuf> {
        #[cfg(unix)]
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            if !xdg_data.is_empty() {
                return Ok(PathBuf::from(xdg_data).join(APP_NAME));
            }
        }

        dirs::data_dir()
            .map(|p| p.join(APP_NAME))
            .context("Could not determine data directory")
    }

    /// Get the XDG cache directory for the application
    ///
    /// Respects `$XDG_CACHE_HOME` on Linux (falls back to `~/.cache`)
    pub fn cache_dir() -> Result<PathBuf> {
        #[cfg(unix)]
        if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
            if !xdg_cache.is_empty() {
                return Ok(PathBuf::from(xdg_cache).join(APP_NAME));
            }
        }

        dirs::cache_dir()
            .map(|p| p.join(APP_NAME))
            .context("Could not determine cache directory")
    }

    /// Get the XDG state directory for the application
    ///
    /// Respects `$XDG_STATE_HOME` on Linux (falls back to `~/.local/state`)
    /// On non-Linux platforms, falls back to data directory
    pub fn state_dir() -> Result<PathBuf> {
        #[cfg(unix)]
        if let Ok(xdg_state) = std::env::var("XDG_STATE_HOME") {
            if !xdg_state.is_empty() {
                return Ok(PathBuf::from(xdg_state).join(APP_NAME));
            }
        }

        // dirs::state_dir() is available on Linux, falls back to data_dir on other platforms
        dirs::state_dir()
            .or_else(dirs::data_dir)
            .map(|p| p.join(APP_NAME))
            .context("Could not determine state directory")
    }

    /// Ensure a directory exists, creating it if necessary
    pub fn ensure_dir(path: &PathBuf) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .with_context(|| format!("Failed to create directory: {:?}", path))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub username: Option<String>,
    pub default_workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultsConfig {
    pub workspace: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub color: bool,
    pub pager: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: true,
            pager: true,
        }
    }
}

impl Config {
    /// Get the configuration directory path (XDG compliant)
    ///
    /// Returns `$XDG_CONFIG_HOME/bitbucket-cli` on Linux,
    /// or platform-appropriate equivalent on other systems.
    pub fn config_dir() -> Result<PathBuf> {
        xdg::config_dir()
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CONFIG_FILE))
    }

    /// Get the data directory path (XDG compliant)
    ///
    /// Returns `$XDG_DATA_HOME/bitbucket-cli` on Linux.
    /// Use this for persistent application data.
    pub fn data_dir() -> Result<PathBuf> {
        xdg::data_dir()
    }

    /// Get the cache directory path (XDG compliant)
    ///
    /// Returns `$XDG_CACHE_HOME/bitbucket-cli` on Linux.
    /// Use this for cached data that can be regenerated.
    pub fn cache_dir() -> Result<PathBuf> {
        xdg::cache_dir()
    }

    /// Get the state directory path (XDG compliant)
    ///
    /// Returns `$XDG_STATE_HOME/bitbucket-cli` on Linux.
    /// Use this for state data like logs and history.
    pub fn state_dir() -> Result<PathBuf> {
        xdg::state_dir()
    }

    /// Load configuration from file, or create default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist (XDG compliant)
        xdg::ensure_dir(&config_dir)?;

        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    /// Set the authenticated username
    pub fn set_username(&mut self, username: &str) {
        self.auth.username = Some(username.to_string());
    }

    /// Get the authenticated username
    pub fn username(&self) -> Option<&str> {
        self.auth.username.as_deref()
    }

    /// Set the default workspace
    pub fn set_default_workspace(&mut self, workspace: &str) {
        self.auth.default_workspace = Some(workspace.to_string());
    }

    /// Get the default workspace
    ///
    /// `[auth] default_workspace` is the primary source; `[defaults] workspace`
    /// remains as a legacy fallback. Remove the fallback (and the
    /// `defaults.workspace` field) in the first release after 1.0.
    pub fn default_workspace(&self) -> Option<&str> {
        self.auth
            .default_workspace
            .as_deref()
            .or(self.defaults.workspace.as_deref())
    }

    /// Clear authentication settings (for logout)
    pub fn clear_auth(&mut self) {
        self.auth.username = None;
        self.auth.default_workspace = None;
        self.defaults.workspace = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.auth.username.is_none());
        assert!(config.display.color);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.display.color, deserialized.display.color);
    }

    #[test]
    fn test_default_workspace_prefers_auth_field() {
        let mut config = Config::default();
        config.auth.default_workspace = Some("auth-ws".to_string());
        config.defaults.workspace = Some("legacy-ws".to_string());
        assert_eq!(config.default_workspace(), Some("auth-ws"));
    }

    #[test]
    fn test_default_workspace_falls_back_to_defaults_field() {
        let mut config = Config::default();
        config.defaults.workspace = Some("legacy-ws".to_string());
        assert_eq!(config.default_workspace(), Some("legacy-ws"));
    }

    #[test]
    fn test_set_default_workspace_writes_auth_field() {
        let mut config = Config::default();
        config.set_default_workspace("my-ws");
        assert_eq!(config.auth.default_workspace.as_deref(), Some("my-ws"));
        assert!(config.defaults.workspace.is_none());
        assert_eq!(config.default_workspace(), Some("my-ws"));
    }

    #[test]
    fn test_clear_auth_clears_default_workspace() {
        let mut config = Config::default();
        config.set_username("someone");
        config.auth.default_workspace = Some("auth-ws".to_string());
        config.defaults.workspace = Some("legacy-ws".to_string());

        config.clear_auth();

        assert!(config.username().is_none());
        assert!(config.default_workspace().is_none());
    }

    #[test]
    fn test_xdg_directories() {
        // These should not panic and should return valid paths
        let config_dir = xdg::config_dir().unwrap();
        let data_dir = xdg::data_dir().unwrap();
        let cache_dir = xdg::cache_dir().unwrap();
        let state_dir = xdg::state_dir().unwrap();

        // All paths should end with our app name
        assert!(config_dir.ends_with("bitbucket-cli"));
        assert!(data_dir.ends_with("bitbucket-cli"));
        assert!(cache_dir.ends_with("bitbucket-cli"));
        assert!(state_dir.ends_with("bitbucket-cli"));
    }

    #[test]
    #[cfg(unix)]
    fn test_xdg_env_override() {
        use std::env;

        // Save original values
        let orig_config = env::var("XDG_CONFIG_HOME").ok();

        // SAFETY: This test runs single-threaded and we restore the original value after
        unsafe {
            // Set custom XDG_CONFIG_HOME
            env::set_var("XDG_CONFIG_HOME", "/tmp/test-xdg-config");
        }

        let config_dir = xdg::config_dir().unwrap();
        assert_eq!(
            config_dir,
            PathBuf::from("/tmp/test-xdg-config/bitbucket-cli")
        );

        // SAFETY: Restoring original environment state
        unsafe {
            match orig_config {
                Some(val) => env::set_var("XDG_CONFIG_HOME", val),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }
}

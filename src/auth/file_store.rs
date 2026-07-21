use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use super::Credential;

/// File-based credential storage (fallback when keyring is unavailable)
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new() -> Result<Self> {
        // NOTE: legacy directory — config.toml lives under "bitbucket-cli"
        // (config::settings::APP_NAME). Existing credential files keep this
        // path until a migration consolidates the two directories.
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("bitbucket");

        // Create config directory if it doesn't exist
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        let path = config_dir.join("credentials.json");

        Ok(Self { path })
    }

    /// Store credentials in a file
    pub fn store_credential(&self, credential: &Credential) -> Result<()> {
        let json =
            serde_json::to_string_pretty(credential).context("Failed to serialize credential")?;

        // Write with restrictive permissions (0600 = read/write for owner only)
        #[cfg(unix)]
        {
            use std::fs::OpenOptions;
            use std::os::unix::fs::OpenOptionsExt;

            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&self.path)
                .and_then(|mut file| {
                    use std::io::Write;
                    file.write_all(json.as_bytes())
                })
                .context("Failed to write credential file")?;
        }

        #[cfg(not(unix))]
        {
            fs::write(&self.path, json).context("Failed to write credential file")?;
        }

        Ok(())
    }

    /// Get credentials from the file
    pub fn get_credential(&self) -> Result<Option<Credential>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&self.path).context("Failed to read credential file")?;

        let credential: Credential =
            serde_json::from_str(&json).context("Failed to parse stored credential")?;

        Ok(Some(credential))
    }

    /// Delete credentials from the file
    pub fn delete_credential(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path).context("Failed to delete credential file")?;
        }
        Ok(())
    }
}

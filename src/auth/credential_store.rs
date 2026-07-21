use anyhow::Result;

use super::{Credential, FileStore, KeyringStore};

/// Credential storage that uses the platform secret store (macOS Keychain,
/// Windows Credential Manager, GNOME Keyring / KDE Wallet), with file-based
/// fallback for migration and headless environments.
pub struct CredentialStore {
    file: FileStore,
    keyring: Option<KeyringStore>,
}

impl CredentialStore {
    pub fn new() -> Result<Self> {
        Ok(Self {
            file: FileStore::new()?,
            keyring: KeyringStore::new().ok(),
        })
    }

    pub fn get_credential(&self) -> Result<Option<Credential>> {
        if let Some(keyring) = &self.keyring {
            if let Some(credential) = keyring.get_credential()? {
                return Ok(Some(credential));
            }

            // Migrate credentials from the legacy plain-text file store.
            // TODO: this migrate-then-delete path is untested — testing it requires seams (trait or
            // injected constructors) over KeyringStore/FileStore so a fake keyring and temp file store can be substituted; see CHANGELOG 0.4.0.
            if let Some(credential) = self.file.get_credential()? {
                keyring.store_credential(&credential)?;
                if let Err(err) = self.file.delete_credential() {
                    eprintln!(
                        "Warning: could not remove legacy plaintext credential file: {}",
                        err
                    );
                }
                return Ok(Some(credential));
            }

            return Ok(None);
        }

        self.file.get_credential()
    }

    pub fn store_credential(&self, credential: &Credential) -> Result<()> {
        if let Some(keyring) = &self.keyring {
            keyring.store_credential(credential)?;
            let _ = self.file.delete_credential();
            return Ok(());
        }

        self.file.store_credential(credential)
    }

    pub fn delete_credential(&self) -> Result<()> {
        if let Some(keyring) = &self.keyring {
            let _ = keyring.delete_credential();
        }

        self.file.delete_credential()
    }
}

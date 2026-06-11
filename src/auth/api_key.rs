use anyhow::{Context, Result};
use dialoguer::Input;

use super::{AuthManager, Credential};

/// Strip bracketed-paste escape markers and control characters that some
/// terminals inject when a token is pasted, then trim surrounding whitespace.
fn sanitize_pasted_token(raw: &str) -> String {
    raw.replace("\x1b[200~", "")
        .replace("\x1b[201~", "")
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}

/// API key authentication flow (fallback method)
/// Note: Atlassian has deprecated app passwords in favor of OAuth2
pub struct ApiKeyAuth;

impl ApiKeyAuth {
    /// Run the API key authentication flow.
    ///
    /// `email` and `api_key` may be supplied up front (via CLI flags or env
    /// vars) to skip the corresponding prompts; when both are present the
    /// flow is fully non-interactive.
    pub async fn authenticate(
        auth_manager: &AuthManager,
        email: Option<String>,
        api_key: Option<String>,
    ) -> Result<Credential> {
        if email.is_none() || api_key.is_none() {
            println!("\n🔐 Bitbucket API Key Authentication");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            println!("⚠️  Note: OAuth 2.0 is the preferred authentication method.");
            println!("   API keys are provided for automation/CI scenarios.");
            println!();
            println!("To create an API key (HTTP access token):");
            println!("1. Go to Bitbucket Settings → Personal settings");
            println!("2. Click 'HTTP access tokens' under 'Access management'");
            println!("3. Click 'Create token'");
            println!("4. Give it a label and select required permissions");
            println!();
        }

        let username: String = match email {
            Some(email) => email,
            None => Input::new()
                .with_prompt("Atlassian account email / Bitbucket username")
                .interact_text()
                .context("Failed to read username")?,
        };

        let api_key = match api_key {
            Some(api_key) => api_key,
            // rpassword reads a full line with echo disabled, so terminal
            // paste works — dialoguer's Password reads key events and drops
            // pasted input in many terminals.
            None => rpassword::prompt_password("API key (HTTP access token): ")
                .context("Failed to read API key")?,
        };

        let api_key = sanitize_pasted_token(&api_key);

        if username.trim().is_empty() {
            anyhow::bail!("Email/username cannot be empty");
        }

        // Validate token format
        if api_key.is_empty() {
            anyhow::bail!("API key cannot be empty");
        }

        // Check for common Atlassian token prefixes
        if !api_key.starts_with("ATATT") && !api_key.starts_with("ATCTT") {
            println!("⚠️  Warning: Token doesn't start with expected prefix (ATATT or ATCTT)");
            println!("   This might not be a valid Bitbucket API token.");
            println!(
                "   Token starts with: {}",
                &api_key.chars().take(5).collect::<String>()
            );
        }

        let credential = Credential::ApiKey {
            username: username.clone(),
            api_key,
        };

        // Validate credentials by making a test API call
        Self::validate_credentials(&credential).await?;

        // Store credentials
        auth_manager.store_credentials(&credential)?;

        println!("\n✅ Successfully authenticated as {}", username);
        println!("💡 Tip: Use 'bitbucket auth login --oauth' for a better experience");

        Ok(credential)
    }

    /// Validate credentials against the Bitbucket API
    async fn validate_credentials(credential: &Credential) -> Result<()> {
        let client = reqwest::Client::new();

        println!("🔍 Validating credentials with Bitbucket API...");

        let response = client
            .get("https://api.bitbucket.org/2.0/user")
            .header("Authorization", credential.auth_header())
            .header("User-Agent", "bitbucket-cli/0.3.0")
            .send()
            .await
            .context("Failed to connect to Bitbucket API")?;

        let status = response.status();

        if status.is_success() {
            Ok(())
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            anyhow::bail!(
                "Authentication failed (401 Unauthorized).\n\n\
                Possible causes:\n\
                - Incorrect username\n\
                - Invalid or expired API token\n\
                - Token doesn't have required permissions\n\n\
                Please verify:\n\
                1. Your Bitbucket username is correct\n\
                2. Your API token is copied completely (should start with 'ATATT' or 'ATCTT')\n\
                3. Token has 'Read' permission at minimum"
            )
        } else {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("<unable to read response>"));
            anyhow::bail!(
                "API error ({}):\n{}\n\n\
                This might indicate:\n\
                - Network connectivity issues\n\
                - Bitbucket API is unavailable\n\
                - Rate limiting",
                status,
                body
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_pasted_token;

    #[test]
    fn passes_through_clean_token() {
        assert_eq!(sanitize_pasted_token("ATATT123abc"), "ATATT123abc");
    }

    #[test]
    fn trims_surrounding_whitespace_and_newline() {
        assert_eq!(sanitize_pasted_token("  ATATT123abc\n"), "ATATT123abc");
    }

    #[test]
    fn strips_bracketed_paste_markers() {
        assert_eq!(
            sanitize_pasted_token("\x1b[200~ATATT123abc\x1b[201~"),
            "ATATT123abc"
        );
    }

    #[test]
    fn strips_stray_control_characters() {
        assert_eq!(sanitize_pasted_token("ATATT\x07123\tabc\r"), "ATATT123abc");
    }
}

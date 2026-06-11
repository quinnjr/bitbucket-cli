use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use dialoguer::{Input, Select};

use crate::auth::{ApiKeyAuth, AuthManager, OAuthFlow};
use crate::config::Config;

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Authenticate with Bitbucket (OAuth 2.0 or API key)
    Login {
        /// Use OAuth 2.0 authentication (interactive browser sign-in)
        #[arg(long, conflicts_with = "api_key")]
        oauth: bool,

        /// Use API key authentication (HTTP access token; for automation/CI)
        #[arg(long, conflicts_with = "oauth")]
        api_key: bool,

        /// Atlassian account email / Bitbucket username (for API key authentication)
        #[arg(long, env = "BITBUCKET_EMAIL", conflicts_with = "oauth")]
        email: Option<String>,

        /// API key (HTTP access token; for API key authentication, implies --api-key)
        #[arg(long, env = "BITBUCKET_API_TOKEN", conflicts_with = "oauth")]
        token: Option<String>,

        /// OAuth Client ID (for OAuth authentication)
        #[arg(long, env = "BITBUCKET_CLIENT_ID")]
        client_id: Option<String>,

        /// OAuth Client Secret (for OAuth authentication)
        #[arg(long, env = "BITBUCKET_CLIENT_SECRET")]
        client_secret: Option<String>,
    },

    /// Remove stored credentials
    Logout,

    /// Show authentication status
    Status,
}

impl AuthCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            AuthCommands::Login {
                oauth,
                api_key,
                email,
                token,
                client_id,
                client_secret,
            } => {
                let auth_manager = AuthManager::new()?;

                let use_api_key = resolve_auth_method(
                    oauth,
                    api_key,
                    email.is_some() || token.is_some(),
                    client_id.is_some() || client_secret.is_some(),
                )?;

                if use_api_key {
                    ApiKeyAuth::authenticate(&auth_manager, email, token).await?;
                    return Ok(());
                }

                // OAuth 2.0 authentication.
                // Resolve consumer credentials from (in priority):
                // 1. CLI flags / env vars
                // 2. Previously stored credentials
                // 3. Interactive prompt (first-time only)
                let stored_consumer = auth_manager.get_credentials().ok().flatten().and_then(|c| {
                    c.oauth_consumer_credentials()
                        .map(|(id, secret)| (id.to_owned(), secret.to_owned()))
                });

                let client_id = client_id
                    .or_else(|| stored_consumer.as_ref().map(|(id, _)| id.clone()))
                    .or_else(|| {
                        println!();
                        println!("📋 OAuth Consumer Setup Required");
                        println!();
                        println!("To use OAuth authentication, create an OAuth consumer in Bitbucket:");
                        println!("1. Go to: https://bitbucket.org/[workspace]/workspace/settings/oauth-consumers/new");
                        println!("2. Set callback URL to ONE of these (pick any available port):");
                        println!("   • http://127.0.0.1:8080/callback");
                        println!("   • http://127.0.0.1:3000/callback");
                        println!("   • http://127.0.0.1:8888/callback");
                        println!("   • http://127.0.0.1:9000/callback");
                        println!("3. Select required permissions:");
                        println!("   ✓ Account (Read)");
                        println!("   ✓ Repositories (Read)");
                        println!("   ✓ Pull requests (Read, Write)");
                        println!("   ✓ Issues (Read, Write)");
                        println!("   ✓ Pipelines (Read, Write)");
                        println!("4. Copy the Key (Client ID) and Secret");
                        println!();

                        Input::<String>::new()
                            .with_prompt("OAuth Client ID (Key)")
                            .interact_text()
                            .ok()
                    })
                    .ok_or_else(|| anyhow::anyhow!("OAuth Client ID is required"))?;

                let client_secret = client_secret
                    .or_else(|| stored_consumer.map(|(_, secret)| secret))
                    .or_else(|| {
                        Input::<String>::new()
                            .with_prompt("OAuth Client Secret")
                            .interact_text()
                            .ok()
                    })
                    .ok_or_else(|| anyhow::anyhow!("OAuth Client Secret is required"))?;

                let oauth = OAuthFlow::new(client_id, client_secret);
                oauth.authenticate(&auth_manager).await?;

                Ok(())
            }

            AuthCommands::Logout => {
                let auth_manager = AuthManager::new()?;
                auth_manager.clear_credentials()?;

                let mut config = Config::load()?;
                config.clear_auth();
                config.save()?;

                println!("{} Logged out successfully", "✓".green());
                Ok(())
            }

            AuthCommands::Status => {
                let auth_manager = AuthManager::new()?;
                let config = Config::load()?;

                if auth_manager.is_authenticated() {
                    println!("{} Authenticated", "✓".green());

                    if let Ok(Some(credential)) = auth_manager.get_credentials() {
                        println!("  {} {}", "Method:".dimmed(), credential.type_name());

                        // Show username from credential for API keys, or config for OAuth
                        if let Some(username) = credential.username() {
                            println!("  {} {}", "Username:".dimmed(), username);
                        } else if let Some(username) = config.username() {
                            println!("  {} {}", "Username:".dimmed(), username);
                        }

                        if credential.needs_refresh() {
                            println!(
                                "  {} {}",
                                "Status:".dimmed(),
                                "Token needs refresh (will auto-refresh on next use)".yellow()
                            );
                        }
                    }

                    if let Some(workspace) = config.default_workspace() {
                        println!("  {} {}", "Workspace:".dimmed(), workspace);
                    }

                    match crate::api::BitbucketClient::from_stored().await {
                        Ok(client) => match client.get::<serde_json::Value>("/user").await {
                            Ok(user) => {
                                if let Some(display_name) = user.get("display_name") {
                                    println!(
                                        "  {} {}",
                                        "Display name:".dimmed(),
                                        display_name.as_str().unwrap_or("Unknown")
                                    );
                                }
                            }
                            Err(e) => {
                                println!("{} Credentials may be invalid: {}", "⚠".yellow(), e);
                            }
                        },
                        Err(e) => {
                            println!("{} Failed to create client: {}", "✗".red(), e);
                        }
                    }
                } else {
                    println!("{} Not authenticated", "✗".red());
                    println!();
                    println!("Run {} to authenticate", "bitbucket auth login".cyan());
                }

                Ok(())
            }
        }
    }
}

/// Resolve which authentication method to use.
///
/// Returns `true` for API key, `false` for OAuth 2.0.
///
/// Priority: explicit flag > method-implying inputs > interactive prompt.
fn resolve_auth_method(
    oauth: bool,
    api_key: bool,
    api_key_inputs_present: bool,
    oauth_inputs_present: bool,
) -> Result<bool> {
    if api_key || api_key_inputs_present {
        return Ok(true);
    }
    if oauth || oauth_inputs_present {
        return Ok(false);
    }

    println!();
    println!("{}", "Choose an authentication method".bold());
    println!();
    println!(
        "  {}  Browser-based sign-in. Recommended for interactive use.",
        "OAuth 2.0".cyan()
    );
    println!(
        "  {}     HTTP access token. For automation, CI, and headless environments.",
        "API key".cyan()
    );
    println!();

    let options = ["OAuth 2.0 (browser sign-in)", "API key (access token)"];

    let selection = Select::new()
        .with_prompt("Authentication method")
        .items(&options)
        .default(0)
        .interact()
        .context("Failed to read authentication method selection")?;

    Ok(selection == 1)
}

#[cfg(test)]
mod tests {
    use super::resolve_auth_method;

    #[test]
    fn explicit_api_key_flag_selects_api_key() {
        assert!(resolve_auth_method(false, true, false, false).unwrap());
    }

    #[test]
    fn explicit_oauth_flag_selects_oauth() {
        assert!(!resolve_auth_method(true, false, false, false).unwrap());
    }

    #[test]
    fn email_or_token_implies_api_key() {
        assert!(resolve_auth_method(false, false, true, false).unwrap());
    }

    #[test]
    fn oauth_inputs_imply_oauth() {
        assert!(!resolve_auth_method(false, false, false, true).unwrap());
    }

    #[test]
    fn api_key_inputs_win_over_oauth_inputs() {
        assert!(resolve_auth_method(false, false, true, true).unwrap());
    }
}

use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{confirm_or_abort, output_json, parse_repo, print_json};
use crate::api::BitbucketClient;
use crate::models::{CreateWebhookRequest, Webhook};

#[derive(Subcommand)]
pub enum WebhookCommands {
    /// List webhooks on a repository
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Create a webhook
    Create {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Destination URL the events are delivered to
        #[arg(long)]
        url: String,

        /// Event key to subscribe to (repeatable, e.g. --event repo:push --event pullrequest:created)
        #[arg(long = "event", value_name = "EVENT", required = true)]
        events: Vec<String>,

        /// Description of the webhook
        #[arg(short, long)]
        description: Option<String>,

        /// Create the webhook in a disabled state
        #[arg(long)]
        inactive: bool,
    },

    /// View webhook details
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Webhook UUID (the brace-wrapped UUID shown by `webhook list`)
        uid: String,
    },

    /// Update a webhook (unset fields keep their current values)
    Update {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Webhook UUID (the brace-wrapped UUID shown by `webhook list`)
        uid: String,

        /// New destination URL
        #[arg(long)]
        url: Option<String>,

        /// Replace the subscribed events (repeatable)
        #[arg(long = "event", value_name = "EVENT")]
        events: Vec<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,

        /// Enable or disable the webhook
        #[arg(long, value_name = "true|false")]
        active: Option<bool>,
    },

    /// Delete a webhook
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Webhook UUID (the brace-wrapped UUID shown by `webhook list`)
        uid: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Tabled)]
struct WebhookRow {
    #[tabled(rename = "UUID")]
    uid: String,
    #[tabled(rename = "URL")]
    url: String,
    #[tabled(rename = "ACTIVE")]
    active: String,
    #[tabled(rename = "EVENTS")]
    events: String,
}

impl WebhookCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            WebhookCommands::List { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let hooks = client
                    .list_webhooks(&workspace, &repo_slug, Some(limit.clamp(1, 100)))
                    .await?;

                if output_json() {
                    return print_json(&hooks.values);
                }

                if hooks.values.is_empty() {
                    println!("No webhooks found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<WebhookRow> = hooks
                    .values
                    .iter()
                    .map(|h| WebhookRow {
                        uid: h.uuid.clone().unwrap_or_default(),
                        url: h.url.clone(),
                        active: if h.active.unwrap_or(false) {
                            "Yes"
                        } else {
                            "No"
                        }
                        .to_string(),
                        events: h.events.join(", ").chars().take(60).collect(),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if hooks.next.is_some() {
                    println!(
                        "\n{} More webhooks available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            WebhookCommands::Create {
                repo,
                url,
                events,
                description,
                inactive,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let request = CreateWebhookRequest {
                    description,
                    url,
                    active: !inactive,
                    events,
                };

                let hook = client
                    .create_webhook(&workspace, &repo_slug, &request)
                    .await?;

                if output_json() {
                    return print_json(&hook);
                }

                println!(
                    "{} Created webhook {} for {}",
                    "✓".green(),
                    hook.uuid.as_deref().unwrap_or("(no uid)").cyan(),
                    hook.url
                );

                Ok(())
            }

            WebhookCommands::View { repo, uid } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let hook = client.get_webhook(&workspace, &repo_slug, &uid).await?;

                if output_json() {
                    return print_json(&hook);
                }

                println!("{}", hook.uuid.as_deref().unwrap_or(&uid).bold());
                println!("{}", "─".repeat(50));

                if let Some(desc) = &hook.description {
                    if !desc.is_empty() {
                        println!("{}", desc);
                        println!();
                    }
                }

                println!("{} {}", "URL:".dimmed(), hook.url.cyan());
                println!(
                    "{} {}",
                    "Active:".dimmed(),
                    if hook.active.unwrap_or(false) {
                        "Yes"
                    } else {
                        "No"
                    }
                );

                if hook.events.is_empty() {
                    println!("{} (none)", "Events:".dimmed());
                } else {
                    println!("{}", "Events:".dimmed());
                    for event in &hook.events {
                        println!("  {}", event);
                    }
                }

                Ok(())
            }

            WebhookCommands::Update {
                repo,
                uid,
                url,
                events,
                description,
                active,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if url.is_none() && events.is_empty() && description.is_none() && active.is_none() {
                    anyhow::bail!(
                        "Nothing to update. Pass at least one option, e.g. --url or --active."
                    );
                }

                let client = BitbucketClient::from_stored().await?;

                // PUT replaces the whole subscription, so merge the requested
                // changes into the current state before sending.
                let current = client.get_webhook(&workspace, &repo_slug, &uid).await?;
                let request = build_webhook_update(current, url, events, description, active);

                let updated = client
                    .update_webhook(&workspace, &repo_slug, &uid, &request)
                    .await?;

                if output_json() {
                    return print_json(&updated);
                }

                println!(
                    "{} Updated webhook {}",
                    "✓".green(),
                    updated.uuid.as_deref().unwrap_or(&uid).cyan()
                );

                Ok(())
            }

            WebhookCommands::Delete { repo, uid, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!("Delete webhook {} from {}?", uid.red(), repo))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client.delete_webhook(&workspace, &repo_slug, &uid).await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Deleted webhook {}", "✓".green(), uid);

                Ok(())
            }
        }
    }
}

/// Merge `webhook update` flags into the current subscription to produce the
/// full replacement body Bitbucket's PUT expects. Unset flags keep the current
/// values; a webhook with no recorded `active` state defaults to enabled.
fn build_webhook_update(
    current: Webhook,
    url: Option<String>,
    events: Vec<String>,
    description: Option<String>,
    active: Option<bool>,
) -> CreateWebhookRequest {
    CreateWebhookRequest {
        description: description.or(current.description),
        url: url.unwrap_or(current.url),
        active: active.or(current.active).unwrap_or(true),
        events: if events.is_empty() {
            current.events
        } else {
            events
        },
    }
}

#[cfg(test)]
mod tests {
    use super::build_webhook_update;
    use crate::models::Webhook;

    fn current() -> Webhook {
        Webhook {
            uuid: Some("{uid}".to_string()),
            url: "https://old.example.com/hook".to_string(),
            description: Some("old description".to_string()),
            active: Some(true),
            events: vec!["repo:push".to_string()],
        }
    }

    #[test]
    fn build_webhook_update_keeps_current_values_when_flags_unset() {
        let request = build_webhook_update(current(), None, Vec::new(), None, None);
        assert_eq!(request.url, "https://old.example.com/hook");
        assert_eq!(request.description.as_deref(), Some("old description"));
        assert!(request.active);
        assert_eq!(request.events, vec!["repo:push".to_string()]);
    }

    #[test]
    fn build_webhook_update_applies_changed_fields() {
        let request = build_webhook_update(
            current(),
            Some("https://new.example.com/hook".to_string()),
            vec!["pullrequest:created".to_string()],
            Some("new description".to_string()),
            Some(false),
        );
        assert_eq!(request.url, "https://new.example.com/hook");
        assert_eq!(request.description.as_deref(), Some("new description"));
        assert!(!request.active);
        assert_eq!(request.events, vec!["pullrequest:created".to_string()]);
    }

    #[test]
    fn build_webhook_update_defaults_unknown_active_state_to_enabled() {
        let mut hook = current();
        hook.active = None;
        let request = build_webhook_update(hook, None, Vec::new(), None, None);
        assert!(request.active);
    }
}

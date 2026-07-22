use anyhow::Result;
use clap::Subcommand;
use tabled::{Table, Tabled};

use crate::api::BitbucketClient;
use crate::models::User;

#[derive(Subcommand)]
pub enum UserCommands {
    /// Show the currently authenticated user
    Whoami,
    /// View a user's public profile
    View {
        /// Account ID or UUID (including braces) of the user
        account: String,
    },
    /// List email addresses on the authenticated account
    Emails,
}

#[derive(Tabled)]
struct EmailRow {
    #[tabled(rename = "EMAIL")]
    email: String,
    #[tabled(rename = "PRIMARY")]
    primary: String,
    #[tabled(rename = "CONFIRMED")]
    confirmed: String,
}

/// Render a user profile as aligned key/value lines.
fn print_user(user: &User) {
    println!("Display name: {}", user.display_name);
    println!("Username:     {}", user.username.as_deref().unwrap_or("-"));
    println!(
        "Account ID:   {}",
        user.account_id.as_deref().unwrap_or("-")
    );
    println!("UUID:         {}", user.uuid);
}

/// Format an optional boolean as Yes/No, with "-" when absent.
fn yes_no(value: Option<bool>) -> String {
    match value {
        Some(true) => "Yes",
        Some(false) => "No",
        None => "-",
    }
    .to_string()
}

impl UserCommands {
    pub async fn run(self) -> Result<()> {
        let client = BitbucketClient::from_stored().await?;

        match self {
            UserCommands::Whoami => {
                let user = client.get_current_user().await?;
                if super::output_json() {
                    return super::print_json(&user);
                }
                print_user(&user);
            }
            UserCommands::View { account } => {
                let user = client.get_user(&account).await?;
                if super::output_json() {
                    return super::print_json(&user);
                }
                print_user(&user);
            }
            UserCommands::Emails => {
                let emails = client.list_user_emails().await?;

                if super::output_json() {
                    return super::print_json(&emails.values);
                }

                if emails.values.is_empty() {
                    println!("No email addresses found");
                    return Ok(());
                }

                let rows: Vec<EmailRow> = emails
                    .values
                    .iter()
                    .map(|e| EmailRow {
                        email: e.email.clone().unwrap_or_else(|| "-".to_string()),
                        primary: yes_no(e.is_primary),
                        confirmed: yes_no(e.is_confirmed),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);
            }
        }

        Ok(())
    }
}

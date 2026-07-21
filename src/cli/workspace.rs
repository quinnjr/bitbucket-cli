use anyhow::Result;
use clap::Subcommand;
use tabled::{Table, Tabled};

use crate::api::BitbucketClient;

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    /// List all workspaces you have access to
    List,
}

#[derive(Tabled)]
struct WorkspaceRow {
    #[tabled(rename = "SLUG")]
    slug: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PRIVATE")]
    private: String,
    #[tabled(rename = "CREATED")]
    created: String,
}

impl WorkspaceCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            WorkspaceCommands::List => {
                let client = BitbucketClient::from_stored().await?;
                let workspaces = client.list_workspaces().await?;

                if workspaces.is_empty() {
                    println!("No workspaces found");
                    return Ok(());
                }

                let rows: Vec<WorkspaceRow> = workspaces
                    .iter()
                    .map(|w| WorkspaceRow {
                        slug: w.slug.clone(),
                        name: w.name.clone(),
                        private: match w.is_private {
                            Some(true) => "Yes",
                            Some(false) => "No",
                            None => "-",
                        }
                        .to_string(),
                        created: w
                            .created_on
                            .map(|d| d.format("%Y-%m-%d").to_string())
                            .unwrap_or_default(),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                Ok(())
            }
        }
    }
}

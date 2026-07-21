use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::future::Future;
use std::io;
use tokio::task::JoinSet;

use super::event::{Event, EventHandler};
use super::ui;
use super::views::{View, ViewState};
use crate::api::BitbucketClient;
use crate::models::{Issue, Paginated, Pipeline, PullRequest, Repository};

/// Application state
pub struct App {
    /// Is the application running
    pub running: bool,
    /// Current view
    pub current_view: View,
    /// View-specific state
    pub view_state: ViewState,
    /// API client
    pub client: Option<BitbucketClient>,
    /// Current workspace
    pub workspace: Option<String>,
    /// Status message
    pub status: Option<String>,
    /// Is loading data
    pub loading: bool,
    /// Error message
    pub error: Option<String>,

    // Data
    pub repositories: Vec<Repository>,
    pub pull_requests: Vec<PullRequest>,
    pub issues: Vec<Issue>,
    pub pipelines: Vec<Pipeline>,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_view: View::Dashboard,
            view_state: ViewState::default(),
            client: None,
            workspace: None,
            status: None,
            loading: false,
            error: None,
            repositories: Vec::new(),
            pull_requests: Vec::new(),
            issues: Vec::new(),
            pipelines: Vec::new(),
        }
    }

    /// Initialize the application with API client
    pub fn with_client(mut self, client: BitbucketClient) -> Self {
        self.client = Some(client);
        self
    }

    /// Set the workspace
    pub fn with_workspace(mut self, workspace: String) -> Self {
        self.workspace = Some(workspace);
        self
    }

    /// Set status message
    pub fn set_status(&mut self, message: &str) {
        self.status = Some(message.to_string());
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status = None;
    }

    /// Set error message
    pub fn set_error(&mut self, message: &str) {
        self.error = Some(message.to_string());
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Switch to a different view
    pub fn switch_view(&mut self, view: View) {
        self.current_view = view;
        self.view_state.selected_index = 0;
        self.clear_error();
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                self.running = false;
                return;
            }
            KeyCode::Char('1') => {
                self.switch_view(View::Dashboard);
                return;
            }
            KeyCode::Char('2') => {
                self.switch_view(View::Repositories);
                return;
            }
            KeyCode::Char('3') => {
                self.switch_view(View::PullRequests);
                return;
            }
            KeyCode::Char('4') => {
                self.switch_view(View::Issues);
                return;
            }
            KeyCode::Char('5') => {
                self.switch_view(View::Pipelines);
                return;
            }
            KeyCode::Esc => {
                self.clear_error();
                return;
            }
            _ => {}
        }

        // View-specific keys
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.view_state.previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = match self.current_view {
                    View::Dashboard => 4,
                    View::Repositories => self.repositories.len(),
                    View::PullRequests => self.pull_requests.len(),
                    View::Issues => self.issues.len(),
                    View::Pipelines => self.pipelines.len(),
                };
                self.view_state.next(max);
            }
            KeyCode::Enter => {
                self.handle_select();
            }
            KeyCode::Char('r') => {
                // Refresh will be handled in main loop
            }
            _ => {}
        }
    }

    /// Handle selection
    fn handle_select(&mut self) {
        match self.current_view {
            View::Dashboard => {
                // Navigate to selected section
                match self.view_state.selected_index {
                    0 => self.switch_view(View::Repositories),
                    1 => self.switch_view(View::PullRequests),
                    2 => self.switch_view(View::Issues),
                    3 => self.switch_view(View::Pipelines),
                    _ => {}
                }
            }
            View::Repositories => {
                if let Some(repo) = self.repositories.get(self.view_state.selected_index) {
                    self.set_status(&format!("Selected: {}", repo.full_name));
                }
            }
            View::PullRequests => {
                if let Some(pr) = self.pull_requests.get(self.view_state.selected_index) {
                    self.set_status(&format!("Selected PR #{}: {}", pr.id, pr.title));
                }
            }
            View::Issues => {
                if let Some(issue) = self.issues.get(self.view_state.selected_index) {
                    self.set_status(&format!("Selected Issue #{}: {}", issue.id, issue.title));
                }
            }
            View::Pipelines => {
                if let Some(pipeline) = self.pipelines.get(self.view_state.selected_index) {
                    self.set_status(&format!("Selected Pipeline #{}", pipeline.build_number));
                }
            }
        }
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Load repositories
    pub async fn load_repositories(&mut self) -> Result<()> {
        if let (Some(client), Some(workspace)) = (&self.client, &self.workspace) {
            self.loading = true;
            match client.list_repositories(workspace, None, Some(50)).await {
                Ok(result) => {
                    self.repositories = result.values;
                    self.clear_error();
                }
                Err(e) => {
                    self.set_error(&format!("Failed to load repositories: {}", e));
                }
            }
            self.loading = false;
        } else {
            self.set_error("No workspace configured");
        }
        Ok(())
    }

    /// Get repository slugs for the workspace, fetching them only when not already loaded
    async fn workspace_repo_slugs(
        &mut self,
        client: &BitbucketClient,
        workspace: &str,
    ) -> Option<Vec<String>> {
        if !self.repositories.is_empty() {
            return Some(
                self.repositories
                    .iter()
                    .map(|repo| repo.slug.clone().unwrap_or_else(|| repo.name.clone()))
                    .collect(),
            );
        }
        match client.list_repositories(workspace, None, Some(50)).await {
            Ok(result) => Some(
                result
                    .values
                    .into_iter()
                    .map(|repo| repo.slug.unwrap_or(repo.name))
                    .collect(),
            ),
            Err(e) => {
                self.set_error(&format!("Failed to load repositories: {}", e));
                None
            }
        }
    }

    /// Fetch paginated data from every repository in parallel.
    ///
    /// Spawns one request per repository slug and drains them, returning the
    /// collected values along with the number of repositories whose request
    /// failed (API error or task panic).
    async fn fetch_from_all_repos<T, F, Fut>(
        client: &BitbucketClient,
        workspace: &str,
        repo_slugs: Vec<String>,
        fetch: F,
    ) -> (Vec<T>, usize)
    where
        F: Fn(BitbucketClient, String, String) -> Fut + Clone + Send + 'static,
        Fut: Future<Output = Result<Paginated<T>>> + Send + 'static,
        T: Send + 'static,
    {
        let mut requests = JoinSet::new();
        for repo_slug in repo_slugs {
            let client = client.clone();
            let workspace = workspace.to_string();
            let fetch = fetch.clone();
            requests.spawn(async move { fetch(client, workspace, repo_slug).await });
        }

        let mut values = Vec::new();
        let mut failures: usize = 0;
        while let Some(result) = requests.join_next().await {
            match result {
                Ok(Ok(page)) => values.extend(page.values),
                Ok(Err(_)) | Err(_) => failures += 1,
            }
        }
        (values, failures)
    }

    /// Sort newest-first, restoring a deterministic order after the
    /// concurrent fetch tasks complete in arbitrary order. Items without a
    /// timestamp (`None`) sort last.
    fn sort_newest_first<T, K: Ord>(items: &mut [T], key: impl Fn(&T) -> K) {
        items.sort_by_key(|item| std::cmp::Reverse(key(item)));
    }

    /// Apply the shared post-fetch error rule: clear the error when at least
    /// one repository succeeded (or there were no repositories to query), but
    /// surface an error when every repository request failed.
    fn apply_fetch_outcome(&mut self, repo_count: usize, failures: usize) {
        if failures > 0 && failures == repo_count {
            self.set_error(&format!(
                "Failed to load data from all {} repositories",
                failures
            ));
        } else {
            self.clear_error();
        }
    }

    /// Load pull requests for the current workspace
    pub async fn load_pull_requests(&mut self) -> Result<()> {
        if let (Some(client), Some(workspace)) = (self.client.clone(), self.workspace.clone()) {
            self.loading = true;
            self.pull_requests.clear();

            // Load PRs from all repositories
            let repo_slugs = match self.workspace_repo_slugs(&client, &workspace).await {
                Some(slugs) => slugs,
                None => {
                    self.loading = false;
                    return Ok(());
                }
            };
            let repo_count = repo_slugs.len();

            let (values, failures) = Self::fetch_from_all_repos(
                &client,
                &workspace,
                repo_slugs,
                |client, workspace, repo_slug| async move {
                    client
                        .list_pull_requests(&workspace, &repo_slug, None, None, Some(10))
                        .await
                },
            )
            .await;
            self.pull_requests.extend(values);
            Self::sort_newest_first(&mut self.pull_requests, |pr| pr.updated_on);

            self.apply_fetch_outcome(repo_count, failures);
            self.loading = false;
        } else {
            self.set_error("No workspace configured");
        }
        Ok(())
    }

    /// Load issues for the current workspace
    pub async fn load_issues(&mut self) -> Result<()> {
        if let (Some(client), Some(workspace)) = (self.client.clone(), self.workspace.clone()) {
            self.loading = true;
            self.issues.clear();

            // Load issues from all repositories
            let repo_slugs = match self.workspace_repo_slugs(&client, &workspace).await {
                Some(slugs) => slugs,
                None => {
                    self.loading = false;
                    return Ok(());
                }
            };

            let repo_count = repo_slugs.len();

            let (values, failures) = Self::fetch_from_all_repos(
                &client,
                &workspace,
                repo_slugs,
                |client, workspace, repo_slug| async move {
                    client
                        .list_issues(&workspace, &repo_slug, None, None, Some(10))
                        .await
                },
            )
            .await;
            self.issues.extend(values);
            Self::sort_newest_first(&mut self.issues, |issue| issue.created_on);

            self.apply_fetch_outcome(repo_count, failures);
            self.loading = false;
        } else {
            self.set_error("No workspace configured");
        }
        Ok(())
    }

    /// Load pipelines for the current workspace
    pub async fn load_pipelines(&mut self) -> Result<()> {
        if let (Some(client), Some(workspace)) = (self.client.clone(), self.workspace.clone()) {
            self.loading = true;
            self.pipelines.clear();

            // Load pipelines from all repositories
            let repo_slugs = match self.workspace_repo_slugs(&client, &workspace).await {
                Some(slugs) => slugs,
                None => {
                    self.loading = false;
                    return Ok(());
                }
            };

            let repo_count = repo_slugs.len();

            let (values, failures) = Self::fetch_from_all_repos(
                &client,
                &workspace,
                repo_slugs,
                |client, workspace, repo_slug| async move {
                    client
                        .list_pipelines(&workspace, &repo_slug, None, Some(10))
                        .await
                },
            )
            .await;
            self.pipelines.extend(values);
            Self::sort_newest_first(&mut self.pipelines, |pipeline| pipeline.created_on);

            self.apply_fetch_outcome(repo_count, failures);
            self.loading = false;
        } else {
            self.set_error("No workspace configured");
        }
        Ok(())
    }

    /// Load all data
    pub async fn load_all_data(&mut self) -> Result<()> {
        self.load_repositories().await?;
        self.load_pull_requests().await?;
        self.load_issues().await?;
        self.load_pipelines().await?;
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the TUI application
pub async fn run_tui(workspace: Option<String>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Try to get API client
    match BitbucketClient::from_stored().await {
        Ok(client) => {
            app = app.with_client(client);
            if let Some(ws) = workspace {
                app = app.with_workspace(ws);
            } else {
                app.set_error("No workspace specified. Use: bitbucket tui --workspace <workspace>");
            }
        }
        Err(e) => {
            app.set_error(&format!("Not authenticated: {}", e));
        }
    }

    // Load initial data if we have a workspace
    if app.workspace.is_some() && app.client.is_some() {
        app.set_status("Loading data...");
        terminal.draw(|f| ui::draw(f, &app))?;

        if let Err(e) = app.load_repositories().await {
            app.set_error(&format!("Failed to load data: {}", e));
        } else {
            app.set_status("Data loaded. Press 'r' to refresh.");
        }
    }

    // Create event handler
    let event_handler = EventHandler::new(250);
    let mut should_refresh = false;

    // Main loop
    while app.running {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &app))?;

        // Handle refresh if requested
        if should_refresh && app.workspace.is_some() && app.client.is_some() {
            should_refresh = false;
            app.set_status("Refreshing...");
            terminal.draw(|f| ui::draw(f, &app))?;

            match app.current_view {
                View::Dashboard | View::Repositories => {
                    let _ = app.load_repositories().await;
                }
                View::PullRequests => {
                    let _ = app.load_pull_requests().await;
                }
                View::Issues => {
                    let _ = app.load_issues().await;
                }
                View::Pipelines => {
                    let _ = app.load_pipelines().await;
                }
            }

            app.set_status("Refreshed");
        }

        // Handle events
        match event_handler.next()? {
            Event::Key(key) => {
                // Check if refresh was requested
                if let crossterm::event::KeyCode::Char('r') = key.code {
                    should_refresh = true;
                }
                app.handle_key(key);
            }
            Event::Tick => {
                // Periodic tick for animations, etc.
            }
            Event::Resize(_, _) => {
                // Terminal will redraw automatically
            }
            _ => {}
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::App;

    #[test]
    fn sort_newest_first_is_deterministic_regardless_of_arrival_order() {
        // Simulates items arriving in arbitrary JoinSet completion order,
        // keyed by Option timestamps as the real models are.
        let mut items: Vec<(Option<i64>, &str)> = vec![
            (Some(2), "middle"),
            (None, "undated"),
            (Some(3), "newest"),
            (Some(1), "oldest"),
        ];
        App::sort_newest_first(&mut items, |item| item.0);
        let order: Vec<&str> = items.iter().map(|item| item.1).collect();
        assert_eq!(order, vec!["newest", "middle", "oldest", "undated"]);
    }
}

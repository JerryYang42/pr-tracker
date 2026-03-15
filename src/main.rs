mod config;
mod fetch;
mod model;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::info;

/// Terminal UI for visualizing active GitHub pull requests.
#[derive(Debug, Parser)]
#[command(name = "pr-tracker", version, about)]
struct Cli {
    /// Path to config file (default: ~/.config/pr-tracker/config.toml).
    #[arg(long, short)]
    config: Option<PathBuf>,

    /// Override repos to fetch (comma-separated "owner/repo" list).
    #[arg(long, value_delimiter = ',')]
    repos: Option<Vec<String>>,

    /// Disable GitHub API fallback even if configured.
    #[arg(long)]
    no_fallback: bool,
}

fn main() -> Result<()> {
    // Initialise tracing — RUST_LOG controls verbosity (default: warn).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    let mut cfg = config::Config::load(cli.config.as_ref())?;

    // CLI flag overrides.
    if let Some(repos) = cli.repos {
        cfg.github.repos = repos;
    }
    if cli.no_fallback {
        cfg.github.use_api_fallback = false;
    }

    info!("Tracking {} repo(s)", cfg.github.repos.len());

    // Fetch and render. 'r' inside the TUI causes run() to return so we
    // re-fetch here; any other exit key also returns Ok(()) and we break.
    loop {
        let results = fetch::fetch_all(&cfg);
        let total_prs: usize = results.iter().map(|r| r.prs.len()).sum();
        info!(
            "Fetched {total_prs} open PR(s) across {} repo(s)",
            results.len()
        );

        let terminal = ratatui::init();
        let outcome = ui::app::run(terminal, results);
        ratatui::restore();

        outcome?;

        // v1: always exit after run() returns (both 'q' and 'r' reach here).
        // A future version can track the exit reason to distinguish refresh vs quit.
        break;
    }

    Ok(())
}

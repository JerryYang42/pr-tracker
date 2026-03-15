use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// List of repositories in "owner/repo" format.
    pub repos: Vec<String>,

    /// Whether to attempt GitHub API fallback when `gh` fails.
    #[serde(default = "default_true")]
    pub use_api_fallback: bool,

    /// Environment variable name holding a GitHub token for API fallback.
    #[serde(default = "default_token_env")]
    pub token_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// How often to auto-refresh in seconds (0 = manual only).
    #[serde(default = "default_refresh")]
    pub refresh_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub github: GitHubConfig,

    #[serde(default)]
    pub ui: UiConfig,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: default_refresh(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_token_env() -> String {
    "GITHUB_TOKEN".to_string()
}

fn default_refresh() -> u64 {
    60
}

impl Config {
    /// Load config from the given path, or from the default XDG location.
    pub fn load(path: Option<&PathBuf>) -> Result<Self> {
        let config_path = match path {
            Some(p) => p.clone(),
            None => default_config_path()?,
        };

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Cannot read config file: {}", config_path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Invalid TOML in config file: {}", config_path.display()))?;

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            !self.github.repos.is_empty(),
            "config.github.repos must list at least one repository"
        );
        for repo in &self.github.repos {
            anyhow::ensure!(
                repo.contains('/'),
                "Each repo must be in 'owner/repo' format, got: {repo}"
            );
        }
        Ok(())
    }
}

fn default_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join("pr-tracker").join("config.toml"))
}

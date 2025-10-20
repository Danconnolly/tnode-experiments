//! Configuration file support for tnode-lab

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Configuration structure for tnode-lab
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Blockchain service endpoint
    pub blockchain_endpoint: Option<String>,

    /// Enable verbose logging
    pub verbose: Option<bool>,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        debug!("Loading configuration from: {}", path.display());

        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        info!("Configuration loaded from: {}", path.display());
        Ok(config)
    }

    /// Find and load configuration from default locations
    ///
    /// Searches in the following order:
    /// 1. ~/.config/tnode/config.yml
    /// 2. ./tnode.yml (current directory)
    pub fn from_default_locations() -> Result<Option<Self>> {
        let default_paths = Self::default_config_paths()?;

        for path in default_paths {
            if path.exists() {
                debug!("Found config file at: {}", path.display());
                return Ok(Some(Self::from_file(&path)?));
            }
        }

        debug!("No default config file found");
        Ok(None)
    }

    /// Get the default configuration file paths
    fn default_config_paths() -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // ~/.config/tnode/config.yml (XDG Base Directory)
        if let Some(home) = home_dir() {
            paths.push(home.join(".config").join("tnode").join("config.yml"));
        }

        // ./tnode.yml (current directory)
        paths.push(PathBuf::from("tnode.yml"));

        Ok(paths)
    }

    /// Merge this config with another, preferring values from `other`
    #[allow(dead_code)]
    pub fn merge(&mut self, other: &Config) {
        if other.blockchain_endpoint.is_some() {
            self.blockchain_endpoint = other.blockchain_endpoint.clone();
        }
        if other.verbose.is_some() {
            self.verbose = other.verbose;
        }
    }
}

/// Get the home directory
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(h) })
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parse() {
        let yaml = r#"
blockchain_endpoint: "127.0.0.1:9000"
verbose: true
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            config.blockchain_endpoint,
            Some("127.0.0.1:9000".to_string())
        );
        assert_eq!(config.verbose, Some(true));
    }

    #[test]
    fn test_config_merge() {
        let mut base = Config {
            blockchain_endpoint: Some("127.0.0.1:8087".to_string()),
            verbose: Some(false),
        };

        let override_config = Config {
            blockchain_endpoint: Some("127.0.0.1:9000".to_string()),
            verbose: None,
        };

        base.merge(&override_config);

        assert_eq!(base.blockchain_endpoint, Some("127.0.0.1:9000".to_string()));
        assert_eq!(base.verbose, Some(false)); // Not overridden
    }
}

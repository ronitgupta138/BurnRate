//! Configuration management and coding agent auto-detection.

use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

/// Global Burnrate configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BurnrateConfig {
    pub default_max_spend: f64,
    pub default_port: u16,
    pub default_host: String,
    pub redact_secrets: bool,
    pub auto_alias: bool,
}

impl Default for BurnrateConfig {
    fn default() -> Self {
        Self {
            default_max_spend: 2.00,
            default_port: 7070,
            default_host: "127.0.0.1".to_string(),
            redact_secrets: true,
            auto_alias: true,
        }
    }
}

impl BurnrateConfig {
    /// Standard config file path: `~/.config/burnrate/config.json`.
    pub fn default_path() -> PathBuf {
        let base = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        base.join(".config").join("burnrate").join("config.json")
    }

    /// Load config from disk or return default.
    pub fn load() -> Self {
        let path = Self::default_path();
        if !path.exists() {
            return Self::default();
        }

        File::open(path)
            .and_then(|mut f| {
                let mut content = String::new();
                f.read_to_string(&mut content)?;
                serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })
            .unwrap_or_default()
    }

    /// Save config to standard location.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        let serialized = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    /// Scan system PATH for known autonomous coding agent binaries.
    pub fn detect_installed_agents() -> Vec<DetectedAgent> {
        let candidate_names = [
            ("claude", "Anthropic Claude Code CLI"),
            ("cursor", "Cursor AI Editor"),
            ("opencode", "OpenCode Autonomous Agent"),
            ("aider", "Aider AI Pair Programmer"),
            ("codex", "OpenAI Codex CLI"),
        ];

        let mut detected = Vec::new();
        for (bin, desc) in candidate_names {
            if let Ok(path) = which(bin) {
                detected.push(DetectedAgent {
                    binary: bin.to_string(),
                    name: desc.to_string(),
                    path: path.to_string_lossy().to_string(),
                });
            }
        }

        detected
    }
}

/// A detected coding agent binary on the local system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAgent {
    pub binary: String,
    pub name: String,
    pub path: String,
}

/// Helper function to locate executable in PATH.
fn which(binary: &str) -> Result<PathBuf, ()> {
    let path_var = std::env::var("PATH").map_err(|_| ())?;
    let separator = if cfg!(windows) { ';' } else { ':' };

    for dir in path_var.split(separator) {
        let candidate = PathBuf::from(dir).join(binary);
        if candidate.is_file() {
            return Ok(candidate);
        }
        #[cfg(windows)]
        {
            let candidate_exe = PathBuf::from(dir).join(format!("{}.exe", binary));
            if candidate_exe.is_file() {
                return Ok(candidate_exe);
            }
        }
    }

    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults_and_serialization() {
        let cfg = BurnrateConfig::default();
        assert_eq!(cfg.default_max_spend, 2.00);
        assert_eq!(cfg.default_port, 7070);
        assert!(cfg.redact_secrets);

        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: BurnrateConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, deserialized);
    }
}

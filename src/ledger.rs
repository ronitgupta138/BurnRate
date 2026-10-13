//! Persistent session ledger for recording and analyzing coding agent sessions.

use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// A single recorded coding agent session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub project_dir: Option<String>,
    pub git_branch: Option<String>,
    pub model: String,
    pub total_spent_usd: f64,
    pub max_spend_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub secrets_redacted: usize,
    pub tripped: bool,
}

impl SessionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model: impl Into<String>,
        total_spent: f64,
        max_spend: f64,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        secrets_redacted: usize,
        tripped: bool,
    ) -> Self {
        let now = Utc::now();
        let id = format!("sess_{}", now.timestamp_millis());
        let (project_dir, git_branch) = Self::detect_environment();

        Self {
            id,
            timestamp: now,
            project_dir,
            git_branch,
            model: model.into(),
            total_spent_usd: total_spent,
            max_spend_usd: max_spend,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            secrets_redacted,
            tripped,
        }
    }

    /// Detect current working directory and git branch if available.
    pub fn detect_environment() -> (Option<String>, Option<String>) {
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        let git_branch = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|out| {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if s.is_empty() || s == "HEAD" {
                        None
                    } else {
                        Some(s)
                    }
                } else {
                    None
                }
            });

        (cwd, git_branch)
    }

    pub fn cache_hit_ratio(&self) -> f64 {
        if self.input_tokens == 0 {
            0.0
        } else {
            (self.cache_read_tokens as f64 / self.input_tokens as f64 * 100.0).min(100.0)
        }
    }
}

/// Aggregated ledger metrics across historical sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LedgerSummary {
    pub total_sessions: usize,
    pub total_spent_usd: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_secrets_redacted: usize,
    pub tripped_sessions: usize,
    pub avg_cache_hit_percent: f64,
}

/// Append-only persistent session ledger.
#[derive(Debug, Clone)]
pub struct Ledger {
    file_path: PathBuf,
}

impl Ledger {
    /// Return the standard default ledger path: `~/.local/share/burnrate/ledger.jsonl`.
    pub fn default_path() -> PathBuf {
        let base = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        base.join(".local").join("share").join("burnrate").join("ledger.jsonl")
    }

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: path.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.file_path
    }

    /// Record a completed session atomically to disk.
    pub fn record_session(&self, record: &SessionRecord) -> Result<(), std::io::Error> {
        if let Some(parent) = self.file_path.parent() {
            create_dir_all(parent)?;
        }

        let serialized = serde_json::to_string(record)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        writeln!(file, "{}", serialized)?;
        file.flush()?;
        Ok(())
    }

    /// Read historical records from disk (newest first).
    pub fn list_records(&self, limit: usize) -> Result<Vec<SessionRecord>, std::io::Error> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);

        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(record) = serde_json::from_str::<SessionRecord>(&line) {
                records.push(record);
            }
        }

        // Newest first
        records.reverse();
        if limit > 0 && records.len() > limit {
            records.truncate(limit);
        }

        Ok(records)
    }

    /// Compute cumulative historical summary across all sessions.
    pub fn summary(&self) -> Result<LedgerSummary, std::io::Error> {
        let records = self.list_records(usize::MAX)?;
        if records.is_empty() {
            return Ok(LedgerSummary::default());
        }

        let mut summary = LedgerSummary {
            total_sessions: records.len(),
            ..Default::default()
        };

        for r in &records {
            summary.total_spent_usd += r.total_spent_usd;
            summary.total_input_tokens += r.input_tokens;
            summary.total_output_tokens += r.output_tokens;
            summary.total_cache_read_tokens += r.cache_read_tokens;
            summary.total_secrets_redacted += r.secrets_redacted;
            if r.tripped {
                summary.tripped_sessions += 1;
            }
        }

        if summary.total_input_tokens > 0 {
            summary.avg_cache_hit_percent = (summary.total_cache_read_tokens as f64
                / summary.total_input_tokens as f64)
                * 100.0;
        }

        Ok(summary)
    }

    /// Format records as a CLI table with colors.
    pub fn render_table(records: &[SessionRecord]) -> String {
        if records.is_empty() {
            return "No historical sessions recorded in ledger yet.".dimmed().to_string();
        }

        let mut out = String::new();
        out.push_str(&format!(
            "┌{0:─<14}┬{0:─<18}┬{0:─<16}┬{0:─<10}┬{0:─<10}┬{0:─<10}┬{0:─<10}┐\n",
            ""
        ));
        out.push_str(&format!(
            "│ {: <12} │ {: <16} │ {: <14} │ {: <8} │ {: <8} │ {: <8} │ {: <8} │\n",
            "SESSION ID".bold(),
            "DATE (UTC)".bold(),
            "MODEL".bold(),
            "SPENT".bold(),
            "CACHE %".bold(),
            "SECRETS".bold(),
            "STATUS".bold()
        ));
        out.push_str(&format!(
            "├{0:─<14}┼{0:─<18}┼{0:─<16}┼{0:─<10}┼{0:─<10}┼{0:─<10}┼{0:─<10}┤\n",
            ""
        ));

        for r in records {
            let date_str = r.timestamp.format("%Y-%m-%d %H:%M").to_string();
            let spent_str = format!("${:.2}", r.total_spent_usd);
            let cache_str = format!("{:.1}%", r.cache_hit_ratio());
            let secrets_str = if r.secrets_redacted > 0 {
                format!("🛡️ {}", r.secrets_redacted).yellow().to_string()
            } else {
                "0".to_string()
            };
            let status_str = if r.tripped {
                "TRIPPED".red().bold().to_string()
            } else {
                "OK".green().to_string()
            };

            out.push_str(&format!(
                "│ {: <12} │ {: <16} │ {: <14} │ {: <8} │ {: <8} │ {: <8} │ {: <8} │\n",
                r.id.chars().take(12).collect::<String>(),
                date_str,
                r.model.chars().take(14).collect::<String>(),
                spent_str,
                cache_str,
                secrets_str,
                status_str
            ));
        }

        out.push_str(&format!(
            "└{0:─<14}┴{0:─<18}┴{0:─<16}┴{0:─<10}┴{0:─<10}┴{0:─<10}┴{0:─<10}┘",
            ""
        ));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_record_and_summary() {
        let temp_dir = std::env::temp_dir().join(format!("burnrate_test_{}", Utc::now().timestamp_nanos_opt().unwrap_or(1)));
        let ledger_path = temp_dir.join("ledger.jsonl");
        let ledger = Ledger::new(&ledger_path);

        let rec1 = SessionRecord::new("claude-sonnet", 0.45, 2.00, 10000, 400, 8000, 1, false);
        let rec2 = SessionRecord::new("claude-opus", 2.05, 2.00, 25000, 800, 15000, 0, true);

        ledger.record_session(&rec1).expect("Record 1 saved");
        ledger.record_session(&rec2).expect("Record 2 saved");

        let loaded = ledger.list_records(10).expect("Loaded records");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, rec2.id); // Newest first

        let summary = ledger.summary().expect("Summary generated");
        assert_eq!(summary.total_sessions, 2);
        assert!((summary.total_spent_usd - 2.50).abs() < 0.01);
        assert_eq!(summary.total_secrets_redacted, 1);
        assert_eq!(summary.tripped_sessions, 1);

        let table = Ledger::render_table(&loaded);
        assert!(table.contains("SESSION ID"));
        assert!(table.contains("TRIPPED"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}

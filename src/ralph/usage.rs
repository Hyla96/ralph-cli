use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TaskUsage {
    #[serde(rename = "inputTokens")]
    pub input_tokens: u64,
    #[serde(rename = "outputTokens")]
    pub output_tokens: u64,
    #[serde(rename = "cacheReadTokens")]
    pub cache_read_tokens: u64,
    #[serde(rename = "cacheWriteTokens")]
    pub cache_write_tokens: u64,
    #[serde(rename = "estimatedCostUsd")]
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UsageFile {
    pub tasks: HashMap<String, TaskUsage>,
    pub total: TaskUsage,
}

impl UsageFile {
    /// Loads `usage.json` from the given directory.
    /// Returns `Ok(UsageFile::default())` if the file does not exist.
    /// Propagates parse errors.
    pub fn load(dir: &Path) -> Result<Self> {
        let usage_path = dir.join("usage.json");
        if !usage_path.exists() {
            return Ok(UsageFile::default());
        }
        let content = std::fs::read_to_string(&usage_path)
            .with_context(|| format!("failed to read {}", usage_path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("failed to parse {}", usage_path.display()))
    }

    /// Saves the usage file as pretty-printed JSON to `usage.json` in the given directory.
    pub fn save(&self, dir: &Path) -> Result<()> {
        let usage_path = dir.join("usage.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&usage_path, json)
            .with_context(|| format!("failed to write {}", usage_path.display()))
    }

    /// Records a task's token usage, updating the total.
    pub fn record_task(&mut self, task_id: &str, usage: TaskUsage) {
        self.tasks.insert(task_id.to_string(), usage);
        // Recompute total as the sum of all entries.
        self.total = TaskUsage::default();
        for entry in self.tasks.values() {
            self.total.input_tokens = self.total.input_tokens.saturating_add(entry.input_tokens);
            self.total.output_tokens = self.total.output_tokens.saturating_add(entry.output_tokens);
            self.total.cache_read_tokens =
                self.total.cache_read_tokens.saturating_add(entry.cache_read_tokens);
            self.total.cache_write_tokens =
                self.total.cache_write_tokens.saturating_add(entry.cache_write_tokens);
            self.total.estimated_cost_usd += entry.estimated_cost_usd;
        }
    }
}

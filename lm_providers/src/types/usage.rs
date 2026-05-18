use serde::{Deserialize, Serialize};

use super::Model;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    #[serde(rename = "cacheRead")]
    pub cache_read: u64,
    #[serde(rename = "cacheWrite")]
    pub cache_write: u64,
    #[serde(rename = "totalTokens")]
    pub total_tokens: u64,
    pub cost: Cost,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Cost {
    pub input: f64,
    pub output: f64,
    #[serde(rename = "cacheRead")]
    pub cache_read: f64,
    #[serde(rename = "cacheWrite")]
    pub cache_write: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    Stop,
    Length,
    ToolUse,
    Error,
    Aborted,
}

impl Usage {
    pub fn calculate_cost(&mut self, model: &Model) {
        self.cost.input = (model.cost.input / 1_000_000.0) * self.input as f64;
        self.cost.output = (model.cost.output / 1_000_000.0) * self.output as f64;
        self.cost.cache_read = (model.cost.cache_read / 1_000_000.0) * self.cache_read as f64;
        self.cost.cache_write = (model.cost.cache_write / 1_000_000.0) * self.cache_write as f64;
        self.cost.total =
            self.cost.input + self.cost.output + self.cost.cache_read + self.cost.cache_write;
    }
}

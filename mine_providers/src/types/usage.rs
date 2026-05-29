use serde::{Deserialize, Serialize};

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

/// Reason why the LLM stopped generating.
///
/// Shared type used by both transport and execution layers to indicate
/// why generation completed (natural stop, length limit, tool use, error, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    /// Natural completion - model finished its response
    Stop,
    /// Hit maximum token length limit
    Length,
    /// Model requested tool execution
    ToolUse,
    /// Error occurred during generation
    Error,
    /// Generation was aborted/cancelled
    Aborted,
}

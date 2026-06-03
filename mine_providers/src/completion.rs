/// Simplified options for LLM completion requests.
///
/// Contains only the essential parameters needed for non-streaming completion.
/// Removed streaming-specific options (transport, cache_retention, session_id, etc.)
/// that were not being used.
#[derive(Debug, Clone, Default)]
pub struct CompletionOptions {
    /// Temperature for sampling (0.0 = deterministic, higher = more random)
    pub temperature: Option<f32>,

    /// Maximum number of tokens to generate
    pub max_tokens: Option<u64>,

    /// Optional metadata to attach to the request
    pub metadata: Option<serde_json::Value>,
}

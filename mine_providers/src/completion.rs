/// Configuration type for model provider API options.
#[derive(Debug, Clone, Default)]
pub struct CompletionOptions {
    /// Temperature for sampling (0.0 = deterministic, higher = more random)
    pub temperature: Option<f32>,

    /// Maximum number of tokens to generate
    pub max_tokens: Option<u64>,

    /// Indicates to the model provider that it has to select a tool to call.
    pub tool_call_only: bool,
}

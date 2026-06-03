use async_trait::async_trait;

use crate::completion::CompletionOptions;
use crate::context::TransportContext;
use crate::error::ProviderError;
use crate::types::{AssistantTransportMessage, Model};

#[async_trait]
pub trait ProviderTrait: Send + Sync {
    fn provider_id(&self) -> &str;

    fn api_id(&self) -> &str;

    /// Complete a request and return the full response.
    ///
    /// Makes a single request to the LLM and returns the complete response.
    async fn complete_direct(
        &self,
        model: &Model,
        context: &TransportContext,
        options: CompletionOptions,
    ) -> Result<AssistantTransportMessage, ProviderError>;

    /// Indicates whether this provider supports `feature`.
    fn supports_feature(&self, _feature: ProviderFeature) -> bool {
        false
    }

    fn models(&self) -> Vec<Model> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFeature {
    ToolCalling,
    Vision,
    Reasoning,
    PromptCaching,
}

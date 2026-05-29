use async_trait::async_trait;
use futures::StreamExt;

use crate::error::ProviderError;
use crate::stream::{TransportContext, EventStream, SimpleStreamOptions, StreamEvent, StreamOptions};
use crate::types::{AssistantMessage, Model};

#[async_trait]
pub trait ProviderTrait: Send + Sync {
    fn provider_id(&self) -> &str;

    fn api_id(&self) -> &str;

    async fn stream(
        &self,
        model: &Model,
        context: &TransportContext,
        options: StreamOptions,
    ) -> Result<EventStream, ProviderError>;

    async fn stream_simple(
        &self,
        model: &Model,
        context: &TransportContext,
        options: SimpleStreamOptions,
    ) -> Result<EventStream, ProviderError> {
        self.stream(model, context, options.base).await
    }

    async fn complete(
        &self,
        model: &Model,
        context: &TransportContext,
        options: StreamOptions,
    ) -> Result<AssistantMessage, ProviderError> {
        dbg!("In complete");
        let mut stream = self.stream(model, context, options).await?;
        let mut final_message = None;

        while let Some(event) = stream.next().await {
            match event? {
                StreamEvent::Done { message, .. } => {
                    final_message = Some(message);
                    break;
                }
                StreamEvent::Error { error, .. } => {
                    return Ok(error);
                }
                _ => {}
            }
        }

        final_message.ok_or(ProviderError::StreamEnded)
    }

    async fn complete_simple(
        &self,
        model: &Model,
        context: &TransportContext,
        options: SimpleStreamOptions,
    ) -> Result<AssistantMessage, ProviderError> {
        self.complete(model, context, options.base).await
    }

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
    Streaming,
    ToolCalling,
    Vision,
    Reasoning,
    PromptCaching,
    WebsocketTransport,
}

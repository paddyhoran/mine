mod definition;
mod registry;
mod trait_def;

#[cfg(feature = "local-candle")]
pub mod local_candle;

#[cfg(feature = "aws-bedrock")]
pub mod bedrock;

#[cfg(feature = "http-client")]
pub mod openai;
pub mod semantics;

use crate::error::ProviderError;
use crate::stream::{EventStream, SimpleStreamOptions, StreamOptions, TransportContext};
use crate::types::{AssistantMessage, Model};
pub use definition::ProviderDefinition;
pub use registry::{global_registry, ProviderRegistry};
use std::sync::Arc;
pub use trait_def::{ProviderFeature, ProviderTrait};

#[cfg(feature = "local-candle")]
pub use local_candle::LocalCandleProvider;

#[cfg(feature = "aws-bedrock")]
pub use bedrock::BedrockProvider;

#[cfg(feature = "http-client")]
pub use openai::OpenAIProvider;

pub struct Provider {
    inner: Arc<dyn ProviderTrait>,
}

impl Provider {
    /// Creates a new provider from a provider definition.
    pub async fn new(definition: ProviderDefinition) -> Result<Self, ProviderError> {
        let inner: Arc<dyn ProviderTrait> = match definition {
            #[cfg(feature = "local-candle")]
            ProviderDefinition::LocalCandle {
                model_repo,
                model_file,
                tokenizer_repo,
            } => Arc::new(LocalCandleProvider::new(
                model_repo,
                model_file,
                tokenizer_repo,
            )?),

            #[cfg(feature = "aws-bedrock")]
            ProviderDefinition::Bedrock { model_id } => {
                Arc::new(BedrockProvider::new(model_id).await?)
            }

            #[cfg(feature = "http-client")]
            ProviderDefinition::OpenAI {
                base_url,
                api_key,
                model_id,
            } => Arc::new(openai::OpenAIProvider::new(base_url, api_key, model_id)?),
        };

        Ok(Self { inner })
    }

    /// Returns the provider ID for this provider.
    pub fn provider_id(&self) -> &str {
        self.inner.provider_id()
    }

    /// Returns the API id for this provider.
    pub fn api_id(&self) -> &str {
        self.inner.api_id()
    }

    pub async fn stream(
        &self,
        model: &Model,
        context: &TransportContext,
        options: StreamOptions,
    ) -> Result<EventStream, ProviderError> {
        self.inner.stream(model, context, options).await
    }

    pub async fn stream_simple(
        &self,
        model: &Model,
        context: &TransportContext,
        options: SimpleStreamOptions,
    ) -> Result<EventStream, ProviderError> {
        self.inner.stream_simple(model, context, options).await
    }

    pub async fn complete(
        &self,
        model: &Model,
        context: &TransportContext,
        options: StreamOptions,
    ) -> Result<AssistantMessage, ProviderError> {
        self.inner.complete(model, context, options).await
    }

    pub async fn complete_simple(
        &self,
        model: &Model,
        context: &TransportContext,
        options: SimpleStreamOptions,
    ) -> Result<AssistantMessage, ProviderError> {
        self.inner.complete_simple(model, context, options).await
    }

    pub fn supports_feature(&self, feature: ProviderFeature) -> bool {
        self.inner.supports_feature(feature)
    }
}

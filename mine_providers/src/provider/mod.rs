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

use crate::context::TransportContext;
use crate::error::ProviderError;
use crate::types::{AssistantTransportMessage, Model};
pub use definition::ProviderDefinition;
pub use registry::{global_registry, ProviderRegistry};
use std::sync::Arc;
pub use trait_def::{ProviderFeature, ProviderTrait};

#[cfg(feature = "local-candle")]
pub use local_candle::LocalCandleProvider;

#[cfg(feature = "aws-bedrock")]
pub use bedrock::BedrockProvider;

use crate::CompletionOptions;
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
            } => Arc::new(OpenAIProvider::new(base_url, api_key, model_id)?),
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

    /// Complete a request and return the full response.
    pub async fn complete_direct(
        &self,
        model: &Model,
        context: &TransportContext,
        options: CompletionOptions,
    ) -> Result<AssistantTransportMessage, ProviderError> {
        self.inner.complete_direct(model, context, options).await
    }

    pub fn supports_feature(&self, feature: ProviderFeature) -> bool {
        self.inner.supports_feature(feature)
    }
}

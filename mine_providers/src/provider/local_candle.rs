use async_trait::async_trait;
use futures::stream;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama::ModelWeights;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

use crate::error::ProviderError;
use crate::provider::ProviderTrait;
use crate::stream::{EventStream, StreamOptions, TransportContext};
use crate::types::{AssistantContent, AssistantMessage, Model, StopReason, Usage};
use crate::InputSemantics;

pub struct LocalCandleProvider {
    model_weights: Arc<Mutex<ModelWeights>>,
    tokenizer: Arc<Tokenizer>,
    device: Device,
}

impl LocalCandleProvider {
    pub fn new(
        model_repo: impl Into<String>,
        model_file: impl Into<String>,
        tokenizer_repo: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let api = Api::new().map_err(|e| ProviderError::Other(format!("HF API error: {}", e)))?;

        let repo = api.repo(Repo::new(model_repo.into(), RepoType::Model));
        let model_path = repo
            .get(&model_file.into())
            .map_err(|e| ProviderError::Other(format!("Failed to download model: {}", e)))?;

        let tokenizer_repo = api.repo(Repo::new(tokenizer_repo.into(), RepoType::Model));
        let tokenizer_path = tokenizer_repo
            .get("tokenizer.json")
            .map_err(|e| ProviderError::Other(format!("Failed to download tokenizer: {}", e)))?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| ProviderError::Other(format!("Failed to load tokenizer: {}", e)))?;

        let device = Device::Cpu;

        let mut file = std::fs::File::open(&model_path)
            .map_err(|e| ProviderError::Other(format!("Failed to open model file: {}", e)))?;

        let model_weights = ModelWeights::from_gguf(
            candle_core::quantized::gguf_file::Content::read(&mut file)
                .map_err(|e| ProviderError::Other(format!("Failed to read GGUF: {}", e)))?,
            &mut file,
            &device,
        )
        .map_err(|e| ProviderError::Other(format!("Failed to load model weights: {}", e)))?;

        Ok(Self {
            model_weights: Arc::new(Mutex::new(model_weights)),
            tokenizer: Arc::new(tokenizer),
            device,
        })
    }

    fn generate_text(&self, prompt: &str, max_tokens: usize) -> Result<String, ProviderError> {
        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| ProviderError::Other(format!("Tokenization error: {}", e)))?;

        let token_ids = tokens.get_ids();
        let mut input_tokens = token_ids.to_vec();

        let mut generated_text = String::new();
        let eos_token_id = InputSemantics::Llama3.eos_token_id();

        for _ in 0..max_tokens {
            let input_tensor = Tensor::new(input_tokens.as_slice(), &self.device)
                .map_err(|e| ProviderError::Other(format!("Tensor error: {}", e)))?
                .unsqueeze(0)
                .map_err(|e| ProviderError::Other(format!("Tensor error: {}", e)))?;

            let logits = self
                .model_weights
                .lock()
                .map_err(|_| ProviderError::Other("Failed to lock model".to_string()))?
                .forward(&input_tensor, 0)
                .map_err(|e| ProviderError::Other(format!("Forward pass error: {}", e)))?;

            let logits = logits
                .squeeze(0)
                .map_err(|e| ProviderError::Other(format!("Squeeze error: {}", e)))?;

            let logits_vec = logits
                .to_vec1::<f32>()
                .map_err(|e| ProviderError::Other(format!("To vec1 error: {}", e)))?;

            let next_token = logits_vec
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx as u32)
                .ok_or_else(|| ProviderError::Other("No tokens found".to_string()))?;

            if next_token == eos_token_id {
                break;
            }

            input_tokens.push(next_token);

            let decoded = self
                .tokenizer
                .decode(&[next_token], false)
                .map_err(|e| ProviderError::Other(format!("Decode error: {}", e)))?;

            generated_text.push_str(&decoded);
        }

        Ok(generated_text)
    }
}

#[async_trait]
impl ProviderTrait for LocalCandleProvider {
    fn provider_id(&self) -> &str {
        "local-candle"
    }

    fn api_id(&self) -> &str {
        "local-candle"
    }

    async fn stream(
        &self,
        model: &Model,
        context: &TransportContext,
        _options: StreamOptions,
    ) -> Result<EventStream, ProviderError> {
        // Build the prompt using Llama 3 input semantics
        let full_prompt = InputSemantics::Llama3.build_prompt(context)?;

        let generated = self.generate_text(&full_prompt, 512)?;

        let message = AssistantMessage {
            content: vec![AssistantContent::Text {
                text: generated,
                text_signature: None,
            }],
            api: self.api_id().to_string(),
            provider: self.provider_id().to_string(),
            model: model.name.clone(),
            response_id: None,
            usage: Usage::default(),
            stop_reason: StopReason::Stop,
            error_message: None,
            timestamp: SystemTime::now(),
        };

        let events = vec![
            Ok(crate::stream::StreamEvent::Start {
                partial: message.clone(),
            }),
            Ok(crate::stream::StreamEvent::Done {
                reason: StopReason::Stop,
                message,
            }),
        ];

        Ok(Box::pin(stream::iter(events)))
    }

    fn supports_feature(&self, feature: crate::provider::ProviderFeature) -> bool {
        matches!(feature, crate::provider::ProviderFeature::Streaming)
    }
}

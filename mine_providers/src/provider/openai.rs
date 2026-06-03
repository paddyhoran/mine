use async_trait::async_trait;
use std::time::SystemTime;

use reqwest::Client;

use crate::context::TransportContext;
use crate::error::ProviderError;
use crate::provider::ProviderTrait;
use crate::types::{AssistantContent, AssistantTransportMessage, Model, StopReason, Usage};
use crate::OpenAIRequestBuilder;

pub struct OpenAIProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model_id: String,
}

impl OpenAIProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model_id: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let client = Client::new();

        Ok(Self {
            client,
            base_url: base_url.into(),
            api_key: api_key.into(),
            model_id: model_id.into(),
        })
    }

    fn build_request_body(
        &self,
        context: &TransportContext,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut builder = OpenAIRequestBuilder::default().add_system_prompt(&context.system_prompt);

        for msg in &context.messages {
            match msg {
                crate::types::TransportMessage::User(user_msg) => {
                    let content = match &user_msg.content {
                        crate::types::UserContent::Text(text) => text.clone(),
                        crate::types::UserContent::Blocks(_) => {
                            return Err(ProviderError::Other(
                                "Content blocks not supported".to_string(),
                            ))
                        }
                    };
                    builder = builder.add_user_message(content);
                }
                crate::types::TransportMessage::Assistant(assistant_msg) => {
                    let text = assistant_msg
                        .content
                        .iter()
                        .filter_map(|c| match c {
                            AssistantContent::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    builder = builder.add_assistant_message(text);
                }
                _ => {}
            }
        }

        Ok(builder.build(&self.model_id, 4096))
    }
}

#[async_trait]
impl ProviderTrait for OpenAIProvider {
    fn provider_id(&self) -> &str {
        "openai-compatible"
    }

    fn api_id(&self) -> &str {
        "openai"
    }

    async fn complete_direct(
        &self,
        model: &Model,
        context: &TransportContext,
        _options: crate::completion::CompletionOptions,
    ) -> Result<AssistantTransportMessage, ProviderError> {
        let body = self.build_request_body(context)?;

        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Other(format!("HTTP request error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProviderError::Other(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(format!("Failed to parse response: {}", e)))?;

        let choice = response_json["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| ProviderError::ParseError("Missing choices array".to_string()))?;

        let text = choice["message"]["content"]
            .as_str()
            .ok_or_else(|| ProviderError::ParseError("Missing content".to_string()))?
            .to_string();

        let usage = Usage {
            input: response_json["usage"]["prompt_tokens"]
                .as_u64()
                .unwrap_or(0),
            output: response_json["usage"]["completion_tokens"]
                .as_u64()
                .unwrap_or(0),
            cache_read: 0,
            cache_write: 0,
            total_tokens: response_json["usage"]["total_tokens"].as_u64().unwrap_or(0),
            cost: Default::default(),
        };

        let stop_reason = match choice["finish_reason"].as_str() {
            Some("stop") => StopReason::Stop,
            Some("length") => StopReason::Length,
            Some("tool_calls") => StopReason::ToolUse,
            _ => StopReason::Stop,
        };

        Ok(AssistantTransportMessage {
            content: vec![AssistantContent::Text {
                text,
                text_signature: None,
            }],
            api: self.api_id().to_string(),
            provider: self.provider_id().to_string(),
            model: model.name.clone(),
            response_id: response_json["id"].as_str().map(|s| s.to_string()),
            usage,
            stop_reason,
            error_message: None,
            timestamp: SystemTime::now(),
        })
    }
}

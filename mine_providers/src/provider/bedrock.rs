use async_trait::async_trait;
use aws_config::BehaviorVersion;
use std::time::SystemTime;

use aws_sdk_bedrockruntime::Client;
use serde_json::json;

use crate::context::TransportContext;
use crate::error::ProviderError;
use crate::provider::ProviderTrait;
use crate::types::{AssistantContent, AssistantTransportMessage, Model, StopReason, Usage};

pub struct BedrockProvider {
    client: Client,
    model_id: String,
}

impl BedrockProvider {
    pub async fn new(model_id: impl Into<String>) -> Result<Self, ProviderError> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            model_id: model_id.into(),
        })
    }

    fn build_request_body(&self, context: &TransportContext) -> Result<String, ProviderError> {
        let mut messages = Vec::new();

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
                    messages.push(json!({
                        "role": "user",
                        "content": content
                    }));
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
                    messages.push(json!({
                        "role": "assistant",
                        "content": text
                    }));
                }
                _ => {}
            }
        }

        let mut body = json!({
            "messages": messages,
            "anthropic_version": "bedrock-2023-05-31",
            "max_tokens": 1024
        });

        if let Some(system) = &context.system_prompt {
            body["system"] = json!(system);
        }

        serde_json::to_string(&body)
            .map_err(|e| ProviderError::Other(format!("Failed to serialize request: {}", e)))
    }
}

#[async_trait]
impl ProviderTrait for BedrockProvider {
    fn provider_id(&self) -> &str {
        "aws-bedrock"
    }

    fn api_id(&self) -> &str {
        "bedrock"
    }

    async fn complete(
        &self,
        model: &Model,
        context: &TransportContext,
        _options: crate::completion::CompletionOptions,
    ) -> Result<AssistantTransportMessage, ProviderError> {
        let body = self.build_request_body(context)?;

        let response = self
            .client
            .invoke_model()
            .model_id(&self.model_id)
            .body(body.into_bytes().into())
            .send()
            .await
            .map_err(|e| ProviderError::Other(format!("Bedrock API error: {}", e)))?;

        let response_body = response.body().as_ref();
        let response_json: serde_json::Value = serde_json::from_slice(response_body)
            .map_err(|e| ProviderError::ParseError(format!("Failed to parse response: {}", e)))?;

        let content = response_json["content"]
            .as_array()
            .ok_or_else(|| ProviderError::ParseError("Missing content array".to_string()))?;

        let text = content
            .iter()
            .filter_map(|c| {
                if c["type"] == "text" {
                    c["text"].as_str()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        let usage = Usage {
            input: response_json["usage"]["input_tokens"].as_u64().unwrap_or(0),
            output: response_json["usage"]["output_tokens"]
                .as_u64()
                .unwrap_or(0),
            cache_read: 0,
            cache_write: 0,
            total_tokens: response_json["usage"]["input_tokens"].as_u64().unwrap_or(0)
                + response_json["usage"]["output_tokens"]
                    .as_u64()
                    .unwrap_or(0),
            cost: Default::default(),
        };

        let stop_reason = match response_json["stop_reason"].as_str() {
            Some("end_turn") => StopReason::Stop,
            Some("max_tokens") => StopReason::Length,
            Some("stop_sequence") => StopReason::Stop,
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

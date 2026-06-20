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

        // Add tools if present
        if !context.tools.is_empty() {
            builder = builder.with_tools(context.tools.clone());
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

    async fn complete(
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

        let message = &choice["message"];
        
        // Parse content - can be text or tool calls
        let mut content = Vec::new();
        
        // Check for text content
        if let Some(text) = message["content"].as_str() {
            if !text.is_empty() {
                content.push(AssistantContent::Text {
                    text: text.to_string(),
                    text_signature: None,
                });
            }
        }
        
        // Check for tool calls
        if let Some(tool_calls_array) = message["tool_calls"].as_array() {
            for tool_call in tool_calls_array {
                if tool_call["type"].as_str() == Some("function") {
                    let function = &tool_call["function"];
                    let id = tool_call["id"]
                        .as_str()
                        .ok_or_else(|| ProviderError::ParseError("Missing tool call id".to_string()))?
                        .to_string();
                    let name = function["name"]
                        .as_str()
                        .ok_or_else(|| ProviderError::ParseError("Missing function name".to_string()))?
                        .to_string();
                    let arguments_str = function["arguments"]
                        .as_str()
                        .ok_or_else(|| ProviderError::ParseError("Missing function arguments".to_string()))?;
                    
                    // Parse arguments JSON string
                    let arguments: serde_json::Value = serde_json::from_str(arguments_str)
                        .map_err(|e| ProviderError::ParseError(format!("Invalid tool arguments JSON: {}", e)))?;
                    
                    content.push(AssistantContent::ToolCall(crate::types::ToolCall {
                        id,
                        name,
                        arguments,
                        thought_signature: None,
                    }));
                }
            }
        }

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
            content,
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

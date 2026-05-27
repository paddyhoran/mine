use serde_json::{json, Value};

pub enum OpenAIRequestMessage {
    System(String),
    Assistant(String),
    User(String),
}

impl OpenAIRequestMessage {
    fn to_json(&self) -> Value {
        match self {
            OpenAIRequestMessage::System(prompt) => json!({
                "role": "system",
                "content": prompt
            }),
            OpenAIRequestMessage::Assistant(message) => json!({
                "role": "assistant",
                "content": message
            }),
            OpenAIRequestMessage::User(message) => json!({
                "role": "user",
                "content": message
            }),
        }
    }
}

#[derive(Default)]
pub struct OpenAIRequestBuilder {
    messages: Vec<OpenAIRequestMessage>,
}

impl OpenAIRequestBuilder {
    /// Adds a system prompt.
    pub fn add_system_prompt(mut self, message: &Option<String>) -> Self {
        if let Some(system) = message {
            assert!(self.messages.is_empty());
            self.messages
                .push(OpenAIRequestMessage::System(system.clone()));
        };
        self
    }

    /// Adds a user message.
    pub fn add_user_message(mut self, message: impl Into<String>) -> Self {
        self.messages
            .push(OpenAIRequestMessage::User(message.into()));
        self
    }

    /// Adds a assistant message.
    pub fn add_assistant_message(mut self, message: impl Into<String>) -> Self {
        self.messages
            .push(OpenAIRequestMessage::Assistant(message.into()));
        self
    }

    pub fn build(self, model_id: impl Into<String>, max_tokens: u64) -> Value {
        let messages: Vec<_> = self.messages.into_iter().map(|m| m.to_json()).collect();
        json!({
            "model": model_id.into(),
            "messages": messages,
            "max_tokens": max_tokens
        })
    }
}

//! Abstrations over input semantics for difference model families.
use crate::error::ProviderError;
use crate::stream::TransportContext;
use crate::types::{AssistantContent, TransportMessage, UserContent};

/// Represents different chat template formats used by language models.
///
/// Each variant encapsulates the special tokens and formatting rules
/// for a specific model family's chat template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSemantics {
    /// Llama 3 chat template format.
    ///
    /// Uses tokens like <|begin_of_text|>, <|start_header_id|>, etc.
    Llama3,

    /// ChatML format used by many models.
    ///
    /// Uses tokens like <|im_start|> and <|im_end|>
    ChatML,

    /// Llama 2 chat template format.
    ///
    /// Uses tokens like <s>, [INST], and <</SYS>>
    Llama2,

    /// Mistral chat template format.
    ///
    /// Uses tokens like <s> and [INST]
    Mistral,
}

impl InputSemantics {
    /// Returns the beginning-of-text token for this format.
    fn begin_of_text(&self) -> &str {
        match self {
            InputSemantics::Llama3 => "<|begin_of_text|>",
            InputSemantics::ChatML => "",
            InputSemantics::Llama2 => "<s>",
            InputSemantics::Mistral => "<s>",
        }
    }

    /// Formats a system prompt with the special tokens and markers that indicates system
    /// prompts to the model.
    fn format_system_prompt(&self, prompt: &str) -> String {
        match self {
            InputSemantics::Llama3 => {
                format!(
                    "<|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|>",
                    prompt
                )
            }
            InputSemantics::ChatML => {
                format!("<|im_start|>system\n{}<|im_end|>\n", prompt)
            }
            InputSemantics::Llama2 => {
                format!("<<SYS>>\n{}\n<</SYS>>\n\n", prompt)
            }
            InputSemantics::Mistral => {
                // Mistral doesn't have explicit system role
                String::new()
            }
        }
    }

    /// Formats a user message inserting the tokens/markers expected by the model.
    fn format_user(&self, content: &str) -> String {
        match self {
            InputSemantics::Llama3 => {
                format!(
                    "<|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|>",
                    content
                )
            }
            InputSemantics::ChatML => {
                format!("<|im_start|>user\n{}<|im_end|>\n", content)
            }
            InputSemantics::Llama2 => {
                format!("[INST] {} [/INST]", content)
            }
            InputSemantics::Mistral => {
                format!("[INST] {} [/INST]", content)
            }
        }
    }

    /// Formats a complete assistant message inserting tokens/markers expected by the model.
    fn format_assistant(&self, content: &str) -> String {
        match self {
            InputSemantics::Llama3 => {
                format!(
                    "<|start_header_id|>assistant<|end_header_id|>\n\n{}<|eot_id|>",
                    content
                )
            }
            InputSemantics::ChatML => {
                format!("<|im_start|>assistant\n{}<|im_end|>\n", content)
            }
            InputSemantics::Llama2 => {
                format!(" {} </s>", content)
            }
            InputSemantics::Mistral => {
                format!(" {} </s>", content)
            }
        }
    }

    /// Formats the start of an assistant turn (for prompting generation).
    fn format_assistant_start(&self) -> String {
        match self {
            InputSemantics::Llama3 => {
                "<|start_header_id|>assistant<|end_header_id|>\n\n".to_string()
            }
            InputSemantics::ChatML => "<|im_start|>assistant\n".to_string(),
            InputSemantics::Llama2 => " ".to_string(),
            InputSemantics::Mistral => " ".to_string(),
        }
    }

    /// Returns the numeric EOS token ID for this format.
    ///
    /// This is used in the generation loop to detect when to stop.
    pub fn eos_token_id(&self) -> u32 {
        match self {
            InputSemantics::Llama3 => 128009,
            InputSemantics::ChatML => 2,
            InputSemantics::Llama2 => 2,
            InputSemantics::Mistral => 2,
        }
    }

    /// Builds a complete prompt from a Context object.
    ///
    /// This method:
    /// 1. Starts with the begin_of_text token
    /// 2. Adds the system prompt if present
    /// 3. Iterates through messages, formatting each according to its role
    /// 4. Ends with format_assistant_start() to prompt the model to generate
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A user message contains unsupported content blocks
    pub fn build_prompt(&self, context: &TransportContext) -> Result<String, ProviderError> {
        let mut prompt = String::from(self.begin_of_text());

        // Add system prompt if present
        if let Some(system) = &context.system_prompt {
            prompt.push_str(&self.format_system_prompt(system));
        }

        // Add messages
        for msg in &context.messages {
            match msg {
                TransportMessage::User(user_msg) => {
                    let content = match &user_msg.content {
                        UserContent::Text(text) => text.clone(),
                        UserContent::Blocks(_) => {
                            return Err(ProviderError::Other(
                                "Content blocks not supported".to_string(),
                            ))
                        }
                    };
                    prompt.push_str(&self.format_user(&content));
                }
                TransportMessage::Assistant(assistant_msg) => {
                    let text = assistant_msg
                        .content
                        .iter()
                        .filter_map(|c| match c {
                            AssistantContent::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    prompt.push_str(&self.format_assistant(&text));
                }
                TransportMessage::ToolResult(_) => {
                    // Tool results are not yet supported in local providers
                    // Skip for now
                }
            }
        }

        // Add assistant start to prompt generation
        prompt.push_str(&self.format_assistant_start());

        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AssistantMessage, StopReason, Usage, UserMessage};
    use std::time::SystemTime;

    #[test]
    fn test_llama3_basic_prompt() {
        let semantics = InputSemantics::Llama3;
        let mut context = TransportContext::new();
        context.system_prompt = Some("You are a helpful assistant.".to_string());
        context.messages = vec![TransportMessage::User(UserMessage {
            content: UserContent::Text("Hello!".to_string()),
            timestamp: SystemTime::now(),
        })];

        let prompt = semantics.build_prompt(&context).unwrap();

        assert!(prompt.starts_with("<|begin_of_text|>"));
        assert!(prompt.contains("<|start_header_id|>system<|end_header_id|>"));
        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("<|start_header_id|>user<|end_header_id|>"));
        assert!(prompt.contains("Hello!"));
        assert!(prompt.ends_with("<|start_header_id|>assistant<|end_header_id|>\n\n"));
    }

    #[test]
    fn test_llama3_eos_token() {
        let semantics = InputSemantics::Llama3;
        assert_eq!(semantics.eos_token_id(), 128009);
    }

    #[test]
    fn test_chatml_format() {
        let semantics = InputSemantics::ChatML;
        assert_eq!(
            semantics.format_user("test"),
            "<|im_start|>user\ntest<|im_end|>\n"
        );
        assert_eq!(
            semantics.format_assistant("response"),
            "<|im_start|>assistant\nresponse<|im_end|>\n"
        );
    }

    #[test]
    fn test_llama2_format() {
        let semantics = InputSemantics::Llama2;
        assert_eq!(semantics.begin_of_text(), "<s>");
        assert_eq!(semantics.format_user("test"), "[INST] test [/INST]");
    }

    #[test]
    fn test_mistral_no_system() {
        let semantics = InputSemantics::Mistral;
        assert_eq!(semantics.format_system_prompt("system prompt"), "");
    }

    #[test]
    fn test_multi_turn_conversation() {
        let semantics = InputSemantics::Llama3;
        let mut context = TransportContext::new();
        context.messages = vec![
            TransportMessage::User(UserMessage {
                content: UserContent::Text("First".to_string()),
                timestamp: SystemTime::now(),
            }),
            TransportMessage::Assistant(AssistantMessage {
                content: vec![AssistantContent::Text {
                    text: "Response".to_string(),
                    text_signature: None,
                }],
                api: "test".to_string(),
                provider: "test".to_string(),
                model: "test".to_string(),
                response_id: None,
                usage: Usage::default(),
                stop_reason: StopReason::Stop,
                error_message: None,
                timestamp: SystemTime::now(),
            }),
            TransportMessage::User(UserMessage {
                content: UserContent::Text("Second".to_string()),
                timestamp: SystemTime::now(),
            }),
        ];

        let prompt = semantics.build_prompt(&context).unwrap();

        assert!(prompt.contains("First"));
        assert!(prompt.contains("Response"));
        assert!(prompt.contains("Second"));
    }
}

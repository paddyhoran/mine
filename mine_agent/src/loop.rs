use crate::types::{now, Content, Context, Message, StopReason};
use mine_lm_providers::stream::Context as LmContext;
use mine_lm_providers::types::{
    AssistantContent, Message as LmMessage, ToolCall, UserContent, UserMessage,
};
use mine_lm_providers::{Model, Provider, StreamOptions};

pub async fn agent_loop(
    prompt: String,
    context: &mut Context,
    provider: &Provider,
    model: &Model,
) -> Result<Vec<Message>, String> {
    let mut new_messages = vec![];

    let user_msg = Message::User {
        content: prompt,
        timestamp: now(),
    };
    context.messages.push(user_msg.clone());
    new_messages.push(user_msg);

    loop {
        let lm_context = build_lm_context(context)?;

        let assistant_response = provider
            .complete(model, &lm_context, StreamOptions::default())
            .await
            .map_err(|e| format!("Provider error: {}", e))?;

        let stop_reason = match assistant_response.stop_reason {
            mine_lm_providers::StopReason::Stop => StopReason::Stop,
            mine_lm_providers::StopReason::Length => StopReason::Stop,
            mine_lm_providers::StopReason::ToolUse => StopReason::ToolUse,
            mine_lm_providers::StopReason::Error => StopReason::Error,
            mine_lm_providers::StopReason::Aborted => StopReason::Aborted,
        };

        let mut content = Vec::new();
        let mut tool_calls = Vec::new();

        for c in &assistant_response.content {
            match c {
                AssistantContent::Text { text, .. } => {
                    content.push(Content::Text { text: text.clone() });
                }
                AssistantContent::ToolCall(tc) => {
                    content.push(Content::ToolCall {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    });
                    tool_calls.push(tc.clone());
                }
                _ => {}
            }
        }

        let assistant_msg = Message::Assistant {
            content,
            stop_reason,
            timestamp: now(),
        };

        context.messages.push(assistant_msg.clone());
        new_messages.push(assistant_msg.clone());

        if stop_reason == StopReason::Error {
            break;
        }

        if tool_calls.is_empty() {
            break;
        }

        for tool_call in tool_calls {
            let tool = context
                .tools
                .iter()
                .find(|t| t.name == tool_call.name)
                .ok_or_else(|| format!("Tool not found: {}", tool_call.name))?;

            let result = (tool.execute)(tool_call.arguments.clone())
                .map_err(|e| format!("Tool execution error: {}", e))?;

            let tool_result_msg = Message::ToolResult {
                tool_call_id: tool_call.id.clone(),
                tool_name: tool_call.name.clone(),
                content: result.content,
                is_error: false,
                timestamp: now(),
            };

            context.messages.push(tool_result_msg.clone());
            new_messages.push(tool_result_msg);
        }
    }

    Ok(new_messages)
}

fn build_lm_context(context: &Context) -> Result<LmContext, String> {
    let mut lm_messages = Vec::new();

    for msg in &context.messages {
        match msg {
            Message::User { content, timestamp } => {
                lm_messages.push(LmMessage::User(UserMessage {
                    content: UserContent::Text(content.clone()),
                    timestamp: std::time::UNIX_EPOCH + std::time::Duration::from_secs(*timestamp),
                }));
            }
            Message::Assistant {
                content,
                stop_reason,
                timestamp,
            } => {
                let mut lm_content = Vec::new();

                for c in content {
                    match c {
                        Content::Text { text } => {
                            lm_content.push(AssistantContent::Text {
                                text: text.clone(),
                                text_signature: None,
                            });
                        }
                        Content::ToolCall {
                            id,
                            name,
                            arguments,
                        } => {
                            lm_content.push(AssistantContent::ToolCall(ToolCall {
                                id: id.clone(),
                                name: name.clone(),
                                arguments: arguments.clone(),
                                thought_signature: None,
                            }));
                        }
                    }
                }

                lm_messages.push(LmMessage::Assistant(
                    mine_lm_providers::types::AssistantMessage {
                        content: lm_content,
                        api: "openai".to_string(),
                        provider: "openai-compatible".to_string(),
                        model: "".to_string(),
                        response_id: None,
                        usage: Default::default(),
                        stop_reason: match stop_reason {
                            StopReason::Stop => mine_lm_providers::StopReason::Stop,
                            StopReason::ToolUse => mine_lm_providers::StopReason::ToolUse,
                            StopReason::Error => mine_lm_providers::StopReason::Error,
                            StopReason::Aborted => mine_lm_providers::StopReason::Aborted,
                        },
                        error_message: None,
                        timestamp: std::time::UNIX_EPOCH
                            + std::time::Duration::from_secs(*timestamp),
                    },
                ));
            }
            Message::ToolResult {
                tool_call_id,
                tool_name,
                content,
                is_error,
                timestamp,
            } => {
                let mut lm_content = Vec::new();

                for c in content {
                    if let Content::Text { text } = c {
                        lm_content.push(mine_lm_providers::types::ContentBlock::Text {
                            text: text.clone(),
                        });
                    }
                }

                lm_messages.push(LmMessage::ToolResult(
                    mine_lm_providers::types::ToolResultMessage {
                        tool_call_id: tool_call_id.clone(),
                        tool_name: tool_name.clone(),
                        content: lm_content,
                        is_error: *is_error,
                        timestamp: std::time::UNIX_EPOCH
                            + std::time::Duration::from_secs(*timestamp),
                        details: None,
                    },
                ));
            }
        }
    }

    let mut lm_context = LmContext::new()
        .with_system_prompt(&context.system_prompt)
        .with_messages(lm_messages);

    let lm_tools: Vec<_> = context
        .tools
        .iter()
        .map(|tool| mine_lm_providers::types::Tool {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: tool.parameters.clone(),
        })
        .collect();

    if !lm_tools.is_empty() {
        lm_context = lm_context.with_tools(lm_tools);
    }

    Ok(lm_context)
}

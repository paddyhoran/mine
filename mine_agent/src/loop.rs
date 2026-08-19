use crate::types::{now, Content, ExecutionContext, ExecutionMessage};
use mine_providers::types::{
    AssistantContent, ToolCall, TransportMessage, UserContent, UserTransportMessage,
};
use mine_providers::{CompletionOptions, Model, Provider, StopReason, TransportContext};

pub async fn agent_loop(
    prompt: String,
    context: &mut ExecutionContext,
    provider: &Provider,
    model: &Model,
) -> Result<Vec<ExecutionMessage>, String> {
    let mut new_messages = vec![];

    let user_msg = ExecutionMessage::User {
        content: prompt,
        timestamp: now(),
    };
    new_messages.push(user_msg.clone());
    context.update_with_new_messages(&[user_msg]);

    loop {
        let lm_context = build_provider_context(context)?;

        let completion_options = CompletionOptions {
            temperature: None,
            max_tokens: None,
            tool_call_only: !context.tools.is_empty(),
        };

        let assistant_response = provider
            .complete(model, &lm_context, completion_options)
            .await
            .map_err(|e| format!("Provider error: {}", e))?;

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
                AssistantContent::Thinking { .. } => {}
            }
        }

        let assistant_msg = ExecutionMessage::Assistant {
            content,
            stop_reason: assistant_response.stop_reason,
            timestamp: now(),
        };

        new_messages.push(assistant_msg.clone());
        context.update_with_new_messages(&[assistant_msg]);

        // If there are tool calls execute them to augment the context and go back
        // through the loop.  If not, exit.
        if matches!(assistant_response.stop_reason, StopReason::ToolUse) {
            for tool_call in tool_calls {
                let tool = context
                    .tools
                    .iter()
                    .find(|t| t.name == tool_call.name)
                    .ok_or_else(|| format!("Tool not found: {}", tool_call.name))?;

                let result = (tool.execute)(tool_call.arguments.clone())
                    .map_err(|e| format!("Tool execution error: {}", e))?;

                let tool_result_msg = ExecutionMessage::ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    tool_name: tool_call.name.clone(),
                    content: result.content,
                    is_error: false,
                    timestamp: now(),
                };
                new_messages.push(tool_result_msg.clone());
                context.update_with_new_messages(&[tool_result_msg]);
            }
        } else {
            break
        };
    }

    Ok(new_messages)
}

/// Converts execution layer context to transport layer context.
///
/// This is the architectural boundary between agent execution (with executable tools)
/// and provider communication (serializable schemas only). The conversion extracts
/// tool schemas and transforms messages for LLM API requests.
fn build_provider_context(context: &ExecutionContext) -> Result<TransportContext, String> {
    let mut lm_messages = Vec::new();

    for msg in context.messages() {
        match msg {
            ExecutionMessage::User { content, timestamp } => {
                lm_messages.push(TransportMessage::User(UserTransportMessage {
                    content: UserContent::Text(content.clone()),
                    timestamp: std::time::UNIX_EPOCH + std::time::Duration::from_secs(*timestamp),
                }));
            }
            ExecutionMessage::Assistant {
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

                lm_messages.push(TransportMessage::Assistant(
                    mine_providers::types::AssistantTransportMessage {
                        content: lm_content,
                        api: "openai".to_string(),
                        provider: "openai-compatible".to_string(),
                        model: "".to_string(),
                        response_id: None,
                        usage: Default::default(),
                        stop_reason: *stop_reason,
                        error_message: None,
                        timestamp: std::time::UNIX_EPOCH
                            + std::time::Duration::from_secs(*timestamp),
                    },
                ));
            }
            ExecutionMessage::ToolResult {
                tool_call_id,
                tool_name,
                content,
                is_error,
                timestamp,
            } => {
                let mut lm_content = Vec::new();

                for c in content {
                    if let Content::Text { text } = c {
                        lm_content
                            .push(mine_providers::types::ContentBlock::Text { text: text.clone() });
                    }
                }

                lm_messages.push(TransportMessage::ToolResult(
                    mine_providers::types::ToolResultTransportMessage {
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

    let mut lm_context = TransportContext::new()
        .with_system_prompt(&context.system_prompt)
        .with_messages(lm_messages);

    let lm_tools: Vec<_> = context
        .tools
        .iter()
        .map(|tool| mine_providers::types::Tool {
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

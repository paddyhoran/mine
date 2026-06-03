use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Execution layer message for agent loop processing.
///
/// Simplified message format focused on agent execution with minimal metadata.
/// Converted to/from `TransportMessage` when communicating with LLM providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecutionMessage {
    User {
        content: String,
        timestamp: u64,
    },
    Assistant {
        content: Vec<Content>,
        stop_reason: mine_providers::StopReason,
        timestamp: u64,
    },
    ToolResult {
        tool_call_id: String,
        tool_name: String,
        content: Vec<Content>,
        is_error: bool,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Content {
    Text {
        text: String,
    },
    ToolCall {
        id: String,
        name: String,
        arguments: Value,
    },
}

/// Executable tool for agent execution.
///
/// Contains both the schema definition and an executable function.
/// Converted to `mine_providers::Tool` (schema only) when sending to LLM providers.
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub execute: Box<dyn Fn(Value) -> Result<ToolResult, String> + Send + Sync>,
}

/// The result of a tool call.
pub struct ToolResult {
    pub content: Vec<Content>,
}

/// Execution layer context for agent loop processing.
///
/// Contains executable tools and simplified messages for agent execution.
/// Converted to `TransportContext` when communicating with LLM providers.
pub struct ExecutionContext {
    pub system_prompt: String,
    messages: Vec<ExecutionMessage>,
    pub tools: Vec<Tool>,
}

impl ExecutionContext {
    pub fn new(system_prompt: impl Into<String>) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            messages: Vec::new(),
            tools: Vec::new(),
        }
    }

    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// The list of messages in the context in chronological order.
    pub fn messages(&self) -> &[ExecutionMessage] {
        &self.messages
    }

    /// Updates the context with a batch of new messages.
    pub fn update_with_new_messages(&mut self, new_messages: &[ExecutionMessage]) {
        self.messages.extend_from_slice(new_messages)
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

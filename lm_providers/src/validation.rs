use jsonschema::Validator;

use crate::error::ProviderError;
use crate::types::{Tool, ToolCall};

pub fn validate_tool_call(
    tools: &[Tool],
    tool_call: &ToolCall,
) -> Result<serde_json::Value, ProviderError> {
    let tool = tools
        .iter()
        .find(|t| t.name == tool_call.name)
        .ok_or_else(|| {
            ProviderError::ToolValidationError(format!("Tool '{}' not found", tool_call.name))
        })?;

    validate_tool_arguments(tool, tool_call)
}

pub fn validate_tool_arguments(
    tool: &Tool,
    tool_call: &ToolCall,
) -> Result<serde_json::Value, ProviderError> {
    let schema = Validator::new(&tool.parameters).map_err(|e| {
        ProviderError::ToolValidationError(format!("Invalid schema: {}", e))
    })?;

    if let Err(error) = schema.validate(&tool_call.arguments) {
        return Err(ProviderError::ToolValidationError(format!(
            "Validation failed for tool '{}':\n{}\n\nReceived arguments:\n{}",
            tool_call.name,
            error,
            serde_json::to_string_pretty(&tool_call.arguments).unwrap()
        )));
    }

    Ok(tool_call.arguments.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ToolBuilder;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    struct TestParams {
        name: String,
        age: u32,
    }

    #[test]
    fn test_validate_tool_call_success() {
        let tool = ToolBuilder::new("test_tool", "A test tool")
            .parameter::<String>("name", "The name", true)
            .parameter::<u32>("age", "The age", true)
            .build();

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({
                "name": "Alice",
                "age": 30
            }),
            thought_signature: None,
        };

        let result = validate_tool_call(&[tool], &tool_call);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_call_missing_field() {
        let tool = ToolBuilder::new("test_tool", "A test tool")
            .parameter::<String>("name", "The name", true)
            .parameter::<u32>("age", "The age", true)
            .build();

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({
                "name": "Alice"
            }),
            thought_signature: None,
        };

        let result = validate_tool_call(&[tool], &tool_call);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_tool_call_not_found() {
        let tool = ToolBuilder::new("test_tool", "A test tool").build();

        let tool_call = ToolCall {
            id: "call_123".to_string(),
            name: "unknown_tool".to_string(),
            arguments: serde_json::json!({}),
            thought_signature: None,
        };

        let result = validate_tool_call(&[tool], &tool_call);
        assert!(matches!(
            result,
            Err(ProviderError::ToolValidationError(_))
        ));
    }
}

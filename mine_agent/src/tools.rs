use crate::types::{Content, Tool, ToolResult};
use serde_json::{json, Value};

mod read;
// pub use read::create_read_tool;

pub fn create_calculator_tool() -> Tool {
    Tool {
        name: "calculator".to_string(),
        description: "Performs basic arithmetic operations (add, subtract, multiply, divide)"
            .to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "The arithmetic operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "The first number"
                },
                "b": {
                    "type": "number",
                    "description": "The second number"
                }
            },
            "required": ["operation", "a", "b"]
        }),
        execute: Box::new(|args: Value| {
            let operation = args["operation"].as_str().ok_or("Missing operation")?;
            let a = args["a"].as_f64().ok_or("Missing or invalid number 'a'")?;
            let b = args["b"].as_f64().ok_or("Missing or invalid number 'b'")?;

            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    a / b
                }
                _ => return Err(format!("Unknown operation: {}", operation)),
            };

            Ok(ToolResult {
                content: vec![Content::Text {
                    text: format!("{}", result),
                }],
            })
        }),
    }
}

pub fn create_echo_tool() -> Tool {
    Tool {
        name: "echo".to_string(),
        description: "Echoes back the provided message".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "The message to echo back"
                }
            },
            "required": ["message"]
        }),
        execute: Box::new(|args: Value| {
            let message = args["message"].as_str().ok_or("Missing message")?;

            Ok(ToolResult {
                content: vec![Content::Text {
                    text: format!("Echo: {}", message),
                }],
            })
        }),
    }
}

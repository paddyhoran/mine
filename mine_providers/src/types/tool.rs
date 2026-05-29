use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Tool schema definition for LLM provider communication.
///
/// Describes a tool's interface (name, description, parameters) without executable code.
/// This is serialized and sent to LLM APIs. For executable tools, see `mine_agent::Tool`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    #[serde(rename = "thoughtSignature", skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
}

pub struct ToolBuilder {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

impl ToolBuilder {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    pub fn parameter<T: JsonSchema>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let schema = schemars::schema_for!(T);
        let name = name.into();

        if let serde_json::Value::Object(ref mut props) = self.parameters {
            if let Some(serde_json::Value::Object(ref mut properties)) = props.get_mut("properties")
            {
                let mut schema_value = serde_json::to_value(&schema).unwrap();
                if let serde_json::Value::Object(ref mut schema_obj) = schema_value {
                    schema_obj.insert(
                        "description".to_string(),
                        serde_json::Value::String(description.into()),
                    );
                }
                properties.insert(name.clone(), schema_value);
            }

            if required {
                if let Some(serde_json::Value::Array(ref mut required_fields)) =
                    props.get_mut("required")
                {
                    required_fields.push(serde_json::Value::String(name));
                }
            }
        }

        self
    }

    pub fn build(self) -> Tool {
        Tool {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
        }
    }
}

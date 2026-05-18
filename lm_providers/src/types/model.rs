use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {

    /// The unique ID of this model.
    pub id: String,

    /// The display name for the model.
    pub name: String,

    /// The name of the API that the model uses.
    pub api: String,

    /// The model provider.
    pub provider: String,

    /// The URL to the model.
    #[serde(rename = "baseUrl")]
    pub base_url: String,

    pub reasoning: bool,

    /// TODO: Why multiple?
    /// The type of input the model will receive. 
    pub input: Vec<InputType>,

    /// The cost of the model.
    pub cost: ModelCost,

    /// The size of the context window.
    #[serde(rename = "contextWindow")]
    pub context_window: u64,

    /// TODO: ?
    /// The maximum number of tokens to generate.
    #[serde(rename = "maxTokens")]
    pub max_tokens: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}


/// The type of the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {

    /// Text input.
    Text,

    /// Image input.
    Image,
}


/// Represents the cost of the model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
    #[serde(rename = "cacheRead")]
    pub cache_read: f64,
    #[serde(rename = "cacheWrite")]
    pub cache_write: f64,
}

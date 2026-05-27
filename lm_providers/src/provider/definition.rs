//! Type used to configure the provider to use.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ProviderDefinition {
    #[cfg(feature = "local-candle")]
    LocalCandle {
        model_repo: String,
        model_file: String,
        tokenizer_repo: String,
    },
    #[cfg(feature = "aws-bedrock")]
    Bedrock { model_id: String },
    #[cfg(feature = "http-client")]
    OpenAI {
        base_url: String,
        api_key: String,
        model_id: String,
    },
}

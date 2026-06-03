pub mod completion;
pub mod context;
pub mod error;
pub mod provider;
pub mod types;
pub mod validation;

pub use completion::CompletionOptions;
pub use context::TransportContext;
pub use error::{ProviderError, RegistryError};
pub use provider::semantics::*;
pub use provider::{
    global_registry, Provider, ProviderDefinition, ProviderFeature, ProviderRegistry, ProviderTrait,
};
pub use types::{
    AssistantContent, AssistantTransportMessage, ContentBlock, Cost, Model, StopReason, Tool,
    ToolBuilder, ToolCall, ToolResultTransportMessage, TransportMessage, Usage, UserContent,
    UserTransportMessage,
};
pub use validation::{validate_tool_arguments, validate_tool_call};

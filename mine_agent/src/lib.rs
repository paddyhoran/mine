pub mod r#loop;
pub mod tools;
pub mod types;

pub use r#loop::agent_loop;
pub use types::{Content, ExecutionContext, ExecutionMessage, Tool, ToolResult};

// Re-export StopReason from provider crate (shared type)
pub use mine_providers::StopReason;

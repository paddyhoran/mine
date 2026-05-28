use crate::types::{Message, Tool};

/// The context of the model.
#[derive(Debug, Clone)]
pub struct Context {
    /// The system prompt for the model.
    pub system_prompt: Option<String>,

    /// Each of the messages.
    pub messages: Vec<Message>,

    /// The tools available to the model that it can delegate tasks to.
    pub tools: Vec<Tool>,
}

impl Context {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self {
            system_prompt: None,
            messages: Vec::new(),
            tools: Vec::new(),
        }
    }

    /// Updates the system prompt.
    ///
    /// This method replaces the existing context.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Updates the current messages.
    ///
    /// This method replaces the existing messages.
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    /// Updates the tools available to the model.
    ///
    /// This method replaces the existing tools available to the model.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    /// Adds a new message the to existing messages in the context.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Adds a new tool to the list of available tools in the context.
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

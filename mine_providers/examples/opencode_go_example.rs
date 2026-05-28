#[cfg(not(feature = "http-client"))]
fn main() {
    eprintln!("This example requires the 'http-client' feature.");
    eprintln!("Run with: cargo run --example opencode_go_example --features http-client");
    std::process::exit(1);
}

#[cfg(feature = "http-client")]
use mine_lm_providers::*;
#[cfg(feature = "http-client")]
use std::time::SystemTime;

#[cfg(feature = "http-client")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OpenCode Go Provider Example ===\n");

    // PLACEHOLDER: Set your OpenCode Go API key via environment variable:
    // export OPENCODE_API_KEY="your-api-key-here"
    //
    // Get your API key from: https://opencode.ai/go

    let api_key =
        std::env::var("OPENCODE_API_KEY").expect("OPENCODE_API_KEY environment variable not set");

    // PLACEHOLDER: Replace with your desired model
    // Available models: deepseek-v4-pro, deepseek-v4-flash, qwen3.5-plus,
    // qwen3.6-plus, kimi-k2.5, kimi-k2.6, glm-5, glm-5.1, etc.
    // See: https://opencode.ai/zen/go/v1/models
    let model = Model {
        name: "DeepSeek V4 Pro".to_string(),
        provider: ProviderDefinition::OpenAI {
            base_url: "https://opencode.ai/zen/go/v1".to_string(),
            api_key: api_key.clone(),
            model_id: "deepseek-v4-pro".to_string(),
        },
    };

    println!("Connecting to OpenCode Go...");
    let provider = Provider::new(model.provider.clone()).await?;
    println!("Connected successfully!\n");

    let context = Context::new()
        .with_system_prompt("You are a helpful coding assistant.")
        .with_messages(vec![Message::User(UserMessage {
            content: UserContent::Text("Write a simple hello world function in Rust.".to_string()),
            timestamp: SystemTime::now(),
        })]);

    println!("Generating response...\n");

    let response = provider
        .complete(&model, &context, StreamOptions::default())
        .await?;

    if let Some(AssistantContent::Text { text, .. }) = response.content.first() {
        println!("Response:\n{}\n", text);
    }

    println!("Usage:");
    println!("  Input tokens: {}", response.usage.input);
    println!("  Output tokens: {}", response.usage.output);
    println!("  Total tokens: {}", response.usage.total_tokens);

    println!("\n=== Example Complete ===");

    Ok(())
}

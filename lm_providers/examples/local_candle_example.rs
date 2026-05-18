use lm_providers::provider::LocalCandleProvider;
use lm_providers::*;
use std::sync::Arc;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Local Candle Provider Example ===\n");
    
    println!("Loading model (this may take a while on first run)...");
    let provider = Arc::new(LocalCandleProvider::new()?);
    println!("Model loaded successfully!\n");
    
    let model = Model {
        id: "llama-3.2-1b-instruct".to_string(),
        name: "Llama 3.2 1B Instruct".to_string(),
        api: "local-candle".to_string(),
        provider: "local-candle".to_string(),
        base_url: "local".to_string(),
        reasoning: false,
        input: vec![InputType::Text],
        cost: ModelCost {
            input: 0.0,
            output: 0.0,
            cache_read: 0.0,
            cache_write: 0.0,
        },
        context_window: 128_000,
        max_tokens: 2048,
        headers: None,
    };
    
    let context = Context::new()
        .with_system_prompt("You are a helpful assistant.")
        .with_messages(vec![Message::User(UserMessage {
            content: UserContent::Text("What is the capital of France?".to_string()),
            timestamp: SystemTime::now(),
        })]);
    
    println!("Generating response...\n");
    
    let response = provider
        .complete(&model, &context, StreamOptions::default())
        .await?;
    
    if let Some(AssistantContent::Text { text, .. }) = response.content.first() {
        println!("Response: {}\n", text);
    }
    
    println!("=== Example Complete ===");
    
    Ok(())
}

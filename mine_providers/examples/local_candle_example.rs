#[cfg(not(feature = "local-candle"))]
fn main() {
    eprintln!("This example requires the 'local-candle' feature.");
    eprintln!("Run with: cargo run --example local_candle_example --features local-candle");
    std::process::exit(1);
}

#[cfg(feature = "local-candle")]
use mine_lm_providers::*;
#[cfg(feature = "local-candle")]
use std::time::SystemTime;

#[cfg(feature = "local-candle")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Local Candle Provider Example ===\n");

    let model = Model {
        name: "Llama 3.2 1B Instruct".to_string(),
        provider: ProviderDefinition::LocalCandle {
            model_repo: "lmstudio-community/Llama-3.2-1B-Instruct-GGUF".to_string(),
            model_file: "Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string(),
            tokenizer_repo: "unsloth/Llama-3.2-1B-Instruct".to_string(),
        },
    };

    println!("Loading model (this may take a while on first run)...");
    let provider = Provider::new(model.provider.clone()).await?;
    println!("Model loaded successfully!\n");

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

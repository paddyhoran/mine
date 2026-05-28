#[cfg(not(feature = "aws-bedrock"))]
fn main() {
    eprintln!("This example requires the 'aws-bedrock' feature.");
    eprintln!("Run with: cargo run --example bedrock_example --features aws-bedrock");
    std::process::exit(1);
}

#[cfg(feature = "aws-bedrock")]
use mine_lm_providers::*;
#[cfg(feature = "aws-bedrock")]
use std::time::SystemTime;

#[cfg(feature = "aws-bedrock")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AWS Bedrock Provider Example ===\n");

    // PLACEHOLDER: Set your AWS credentials via environment variables:
    // export AWS_ACCESS_KEY_ID="your-access-key"
    // export AWS_SECRET_ACCESS_KEY="your-secret-key"
    // export AWS_REGION="us-east-1"  (or your preferred region)

    // PLACEHOLDER: Replace with your desired Bedrock model ID
    // Examples:
    // - "anthropic.claude-3-5-sonnet-20241022-v2:0"
    // - "anthropic.claude-3-haiku-20240307-v1:0"
    // - "anthropic.claude-v2"
    let model = Model {
        name: "Claude 3.5 Sonnet".to_string(),
        provider: ProviderDefinition::Bedrock {
            model_id: "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
        },
    };

    println!("Connecting to AWS Bedrock...");
    let provider = Provider::new(model.provider.clone()).await?;
    println!("Connected successfully!\n");

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

    println!("Usage:");
    println!("  Input tokens: {}", response.usage.input);
    println!("  Output tokens: {}", response.usage.output);
    println!("  Total tokens: {}", response.usage.total_tokens);

    println!("\n=== Example Complete ===");

    Ok(())
}

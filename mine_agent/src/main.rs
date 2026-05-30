mod r#loop;
mod tools;
mod types;

use clap::Parser;
use mine_providers::{Model, Provider, ProviderDefinition};
use types::{Content, ExecutionContext, ExecutionMessage};

#[derive(Parser, Debug)]
#[command(name = "mine_agent")]
#[command(about = "A simple LLM mine_agent with tool execution", long_about = None)]
struct Args {
    #[arg(help = "The user prompt to send to the mine_agent")]
    prompt: String,

    #[arg(
        short,
        long,
        help = "OpenCode API key (or set OPENCODE_API_KEY env var)"
    )]
    api_key: Option<String>,

    #[arg(
        short,
        long,
        default_value = "https://opencode.ai/zen/go/v1",
        help = "OpenCode API base URL (or set OPENCODE_BASE_URL env var)"
    )]
    base_url: String,

    #[arg(
        short,
        long,
        default_value = "deepseek-v4-pro",
        help = "Model ID to use (or set OPENCODE_MODEL env var)"
    )]
    model: String,

    #[arg(
        short,
        long,
        default_value = "You are a helpful assistant with access to tools.",
        help = "System prompt"
    )]
    system: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let api_key = args
        .api_key
        .or_else(|| std::env::var("OPENCODE_API_KEY").ok())
        .ok_or("API key required. Set OPENCODE_API_KEY or use --api-key")?;

    println!("=== Agent Starting ===\n");
    println!("Model: {}", args.model);
    println!("Base URL: {}", args.base_url);
    println!("System: {}\n", args.system);

    let model = Model {
        name: args.model.clone(),
        provider: ProviderDefinition::OpenAI {
            base_url: args.base_url.clone(),
            api_key: api_key.clone(),
            model_id: args.model.clone(),
        },
    };

    let provider = Provider::new(model.provider.clone()).await?;

    let mut context = ExecutionContext::new(args.system)
        .with_tool(tools::create_calculator_tool())
        .with_tool(tools::create_echo_tool());

    println!("User: {}\n", args.prompt);

    let messages = r#loop::agent_loop(args.prompt, &mut context, &provider, &model)
        .await
        .map_err(|e| format!("Agent loop error: {}", e))?;

    for msg in messages {
        print_message(&msg);
    }

    println!("\n=== Agent Complete ===");

    Ok(())
}

fn print_message(msg: &ExecutionMessage) {
    match msg {
        ExecutionMessage::User { content, .. } => {
            println!("User: {}", content);
        }
        ExecutionMessage::Assistant {
            content,
            stop_reason,
            ..
        } => {
            println!("Assistant:");
            for c in content {
                match c {
                    Content::Text { text } => {
                        println!("  {}", text);
                    }
                    Content::ToolCall {
                        id,
                        name,
                        arguments,
                    } => {
                        println!("  [Tool Call: {} ({})]", name, id);
                        println!(
                            "  Arguments: {}",
                            serde_json::to_string_pretty(arguments).unwrap()
                        );
                    }
                }
            }
            println!("  (Stop reason: {:?})", stop_reason);
        }
        ExecutionMessage::ToolResult {
            tool_name,
            content,
            is_error,
            ..
        } => {
            println!("Tool Result [{}]:", tool_name);
            for c in content {
                if let Content::Text { text } = c {
                    println!("  {}", text);
                }
            }
            if *is_error {
                println!("  (Error)");
            }
        }
    }
    println!();
}

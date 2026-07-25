use mine_agent::tools::create_read_tool;
use serde_json::json;

fn main() {
    let tool = create_read_tool();
    
    println!("Tool name: {}", tool.name);
    println!("Tool description: {}", tool.description);
    println!("Tool parameters: {}", serde_json::to_string_pretty(&tool.parameters).unwrap());
    
    // Test reading a file
    let args = json!({
        "path": "README.md"
    });
    
    println!("\nTesting with args: {}", serde_json::to_string_pretty(&args).unwrap());
    
    match (tool.execute)(args) {
        Ok(result) => {
            println!("\nSuccess!");
            println!("Result content count: {}", result.content.len());
            for content in result.content {
                match content {
                    mine_agent::types::Content::Text { text } => {
                        let preview = if text.len() > 200 {
                            format!("{}...", &text[..200])
                        } else {
                            text
                        };
                        println!("Text content: {}", preview);
                    }
                    _ => println!("Other content type"),
                }
            }
        }
        Err(e) => {
            println!("\nError: {}", e);
        }
    }
}

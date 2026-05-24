//! Example: Using the pi-terraphim-router for intelligent model selection.
//!
//! Run with: cargo run --example terraphim_router --features terraphim-routing

use pi::pi_terraphim_router::{RouterInput, extract_capabilities, route_and_execute};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== pi-terraphim-router Example ===\n");

    // Example 1: Basic routing
    println!("1. Basic routing:");
    let input = RouterInput::new("Implement a secure authentication system with JWT tokens");
    match route_and_execute(input).await {
        Ok(output) => {
            println!("   Provider: {}", output.provider);
            println!("   Model: {}", output.model);
            println!("   Capabilities: {:?}", output.capabilities);
            println!("   Confidence: {}", output.confidence);
            println!(
                "   Response preview: {}...",
                &output.response[..100.min(output.response.len())]
            );
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }

    // Example 2: Extract capabilities without executing
    println!("\n2. Capability extraction:");
    let prompts = vec![
        "Implement a function to parse JSON",
        "Audit this code for security vulnerabilities",
        "Think carefully about this complex algorithm",
        "Write tests for the authentication module",
        "Refactor this messy code",
    ];

    for prompt in prompts {
        let caps = extract_capabilities(prompt);
        println!("   '{}' -> {:?}", prompt, caps);
    }

    // Example 3: With explicit provider override
    println!("\n3. With provider override:");
    let input = RouterInput::new("Explain Rust's borrow checker")
        .with_provider("anthropic")
        .with_model("claude-sonnet-4-6");
    match route_and_execute(input).await {
        Ok(output) => {
            println!("   Provider: {}", output.provider);
            println!("   Model: {}", output.model);
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }

    // Example 4: With system prompt
    println!("\n4. With system prompt:");
    let input = RouterInput::new("Design a microservices architecture")
        .with_system_prompt("You are a senior architect with 20 years of experience");
    match route_and_execute(input).await {
        Ok(output) => {
            println!("   Provider: {}", output.provider);
            println!("   Model: {}", output.model);
            println!("   Capabilities: {:?}", output.capabilities);
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }

    Ok(())
}

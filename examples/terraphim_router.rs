//! Example: Using the pi-terraphim-router for intelligent model selection.
//!
//! Run with: `cargo run --example terraphim_router --features terraphim-routing`.

use pi::pi_terraphim_router::{extract_capabilities, get_provider_for_capability};

fn main() {
    println!("=== pi-terraphim-router Example ===\n");

    println!("1. Capability extraction and provider routing:");
    let prompts = vec![
        "Think carefully about this complex algorithm",
        "Audit this code for security vulnerabilities",
        "Write tests for the authentication module",
        "Design a microservices architecture",
    ];

    for prompt in prompts {
        let caps = extract_capabilities(prompt);
        println!("   Prompt: {prompt}");
        println!("   Capabilities: {caps:?}");
        for cap in caps {
            if let Some(selection) = get_provider_for_capability(&cap) {
                println!(
                    "   Route: {cap} -> {}/{} ({:.2})",
                    selection.provider, selection.model, selection.confidence
                );
            }
        }
        println!();
    }
}

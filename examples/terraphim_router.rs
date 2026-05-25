use pi::pi_terraphim_router::Router;

fn main() {
    println!("=== pi-terraphim-router Example (KG Routing) ===\n");

    let router = Router::default_router().expect("failed to load router");
    println!("Loaded {} routing rules\n", router.rule_count());

    let prompts = vec![
        "implement a secure authentication system",
        "create a plan for the new architecture",
        "verify and validate the test results",
        "build the REST API endpoints",
        "audit this code for security vulnerabilities",
    ];

    for prompt in prompts {
        match router.route(prompt) {
            Some(decision) => {
                println!("Prompt: {prompt}");
                println!(
                    "  Route: {}/{} (concept: {}, priority: {}, confidence: {:.2})",
                    decision.provider,
                    decision.model,
                    decision.matched_concept,
                    decision.priority,
                    decision.confidence,
                );
                if let Some(action) = decision.render_action(prompt) {
                    println!("  Action: {action}");
                }
                println!();
            }
            None => {
                println!("Prompt: {prompt}");
                println!("  No route matched\n");
            }
        }
    }
}

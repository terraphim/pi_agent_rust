//! Terraphim router integration for pi-rust.
//!
//! This module provides intelligent model selection based on keyword-based
//! capability extraction from user prompts. It uses Terraphim's routing engine
//! to match prompt intent to the optimal pi-rust provider and model.
//!
//! # Example
//!
//! ```no_run
//! use pi::terraphim_router::{RouterInput, route_and_execute};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let input = RouterInput::new("Implement a secure authentication system");
//! let output = route_and_execute(input).await?;
//! println!("Provider: {}\nModel: {}\nResponse: {}",
//!     output.provider, output.model, output.response);
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod extractor;
pub mod mapper;
pub mod rpc_client;
pub mod types;

pub use error::{RouterError, RouterResult};
pub use types::{ProviderSelection, RouterInput, RouterOutput};

use crate::pi_terraphim_router::{
    extractor::CapabilityExtractor, mapper::ProviderMapper, rpc_client::RpcClient,
};

/// Route a prompt to the optimal pi-rust provider and return the LLM response.
///
/// # Arguments
/// * `input` - Router input with prompt and optional overrides
///
/// # Returns
/// Structured output with LLM response and routing metadata
///
/// # Errors
/// Returns `RouterError::NoProviderFound` if no provider matches capabilities
/// Returns `RouterError::RpcError` if pi-rust subprocess fails
pub async fn route_and_execute(input: RouterInput) -> RouterResult<RouterOutput> {
    // If preferred provider/model is specified, bypass routing
    if let (Some(provider), Some(model)) = (&input.preferred_provider, &input.preferred_model) {
        let mut client = RpcClient::spawn(provider, model, input.working_dir.as_deref())?;
        let response = client
            .send_prompt(&input.prompt, input.system_prompt.as_deref())
            .await?;
        return Ok(RouterOutput {
            response,
            provider: provider.clone(),
            model: model.clone(),
            capabilities: vec![],
            confidence: 1.0,
            reason: "explicit preference".to_string(),
            fallback_used: false,
        });
    }

    // Extract capabilities from prompt
    let extractor = CapabilityExtractor::new();
    let capabilities = extractor.extract(&input.prompt);

    if capabilities.is_empty() {
        // Fallback to default provider
        let fallback = ProviderMapper::fallback_selection();
        let mut client = RpcClient::spawn(
            &fallback.provider,
            &fallback.model,
            input.working_dir.as_deref(),
        )?;
        let response = client
            .send_prompt(&input.prompt, input.system_prompt.as_deref())
            .await?;
        return Ok(RouterOutput {
            response,
            provider: fallback.provider,
            model: fallback.model,
            capabilities: vec![],
            confidence: 0.0,
            reason: "fallback: no capabilities extracted".to_string(),
            fallback_used: true,
        });
    }

    // Map capabilities to provider
    let mapper = ProviderMapper::new();
    let selection = mapper
        .map(&capabilities)
        .unwrap_or_else(ProviderMapper::fallback_selection);

    // Spawn pi-rust and send prompt
    let mut client = RpcClient::spawn(
        &selection.provider,
        &selection.model,
        input.working_dir.as_deref(),
    )?;
    let response = client
        .send_prompt(&input.prompt, input.system_prompt.as_deref())
        .await?;

    Ok(RouterOutput {
        response,
        provider: selection.provider,
        model: selection.model,
        capabilities: capabilities.iter().map(|c| format!("{c:?}")).collect(),
        confidence: selection.confidence,
        reason: "capability match".to_string(),
        fallback_used: false,
    })
}

/// Extract capabilities from a prompt without executing.
///
/// # Arguments
/// * `prompt` - User prompt text
///
/// # Returns
/// List of extracted capabilities as strings
pub fn extract_capabilities(prompt: &str) -> Vec<String> {
    let extractor = CapabilityExtractor::new();
    extractor
        .extract(prompt)
        .into_iter()
        .map(|c| format!("{c:?}"))
        .collect()
}

/// Get the provider mapping for a given capability.
///
/// # Arguments
/// * `capability` - Capability name (e.g., "DeepThinking")
///
/// # Returns
/// Provider selection with provider name, model, and confidence
pub fn get_provider_for_capability(capability: &str) -> Option<ProviderSelection> {
    let mapper = ProviderMapper::new();
    mapper.get_by_name(capability)
}

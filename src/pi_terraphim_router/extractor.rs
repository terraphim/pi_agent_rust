//! Capability extraction from prompts using Terraphim's keyword router.

use terraphim_router::keyword::KeywordRouter;
use terraphim_types::capability::Capability;

/// Extracts capabilities from user prompts.
pub struct CapabilityExtractor {
    router: KeywordRouter,
}

impl CapabilityExtractor {
    /// Create a new capability extractor with default keyword mappings.
    pub fn new() -> Self {
        Self {
            router: KeywordRouter::new(),
        }
    }

    /// Extract capabilities from a prompt.
    ///
    /// # Arguments
    /// * `prompt` - User prompt text
    ///
    /// # Returns
    /// List of extracted capabilities
    pub fn extract(&self, prompt: &str) -> Vec<Capability> {
        self.router.extract_capabilities(prompt)
    }

    /// Check if a prompt contains any capability-indicating keywords.
    ///
    /// # Arguments
    /// * `prompt` - User prompt text
    ///
    /// # Returns
    /// true if any capability keywords are found
    pub fn has_capabilities(&self, prompt: &str) -> bool {
        self.router.has_keywords(prompt)
    }
}

impl Default for CapabilityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_deep_thinking() {
        let extractor = CapabilityExtractor::new();
        let caps = extractor.extract("I need you to think carefully about this complex problem");
        assert!(caps.contains(&Capability::DeepThinking));
    }

    #[test]
    fn test_extract_code_generation() {
        let extractor = CapabilityExtractor::new();
        let caps = extractor.extract("Please implement a function to parse JSON");
        assert!(caps.contains(&Capability::CodeGeneration));
    }

    #[test]
    fn test_extract_security_audit() {
        let extractor = CapabilityExtractor::new();
        let caps = extractor.extract("Audit this code for security vulnerabilities");
        assert!(caps.contains(&Capability::SecurityAudit));
    }

    #[test]
    fn test_extract_multiple_capabilities() {
        let extractor = CapabilityExtractor::new();
        let caps =
            extractor.extract("Implement a secure authentication system and write tests for it");
        assert!(caps.contains(&Capability::CodeGeneration));
        assert!(caps.contains(&Capability::SecurityAudit));
        assert!(caps.contains(&Capability::Testing));
    }

    #[test]
    fn test_no_capabilities() {
        let extractor = CapabilityExtractor::new();
        let caps = extractor.extract("Hello, how are you today?");
        assert!(caps.is_empty());
    }

    #[test]
    fn test_has_capabilities() {
        let extractor = CapabilityExtractor::new();
        assert!(extractor.has_capabilities("Think about this problem"));
        assert!(!extractor.has_capabilities("Hello world"));
    }
}

//! RPC client for communicating with pi-rust subprocess.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::pi_terraphim_router::error::{RouterError, RouterResult};

/// Client for communicating with pi-rust via JSON-RPC over stdio.
pub struct RpcClient {
    child: std::process::Child,
}

impl RpcClient {
    /// Spawn a new pi-rust subprocess in RPC mode.
    ///
    /// # Arguments
    /// * `provider` - Provider ID (e.g., "anthropic")
    /// * `model` - Model ID (e.g., "claude-sonnet-4-6")
    /// * `working_dir` - Optional working directory
    ///
    /// # Returns
    /// New RPC client connected to pi-rust subprocess
    pub fn spawn(provider: &str, model: &str, working_dir: Option<&Path>) -> RouterResult<Self> {
        let mut cmd = Command::new("pi");
        cmd.arg("--mode")
            .arg("rpc")
            .arg("--provider")
            .arg(provider)
            .arg("--model")
            .arg(model)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let child = cmd.spawn().map_err(|e| {
            RouterError::SubprocessError(format!(
                "failed to spawn pi-rust (provider={provider}, model={model}): {e}"
            ))
        })?;

        Ok(Self { child })
    }

    /// Send a prompt to pi-rust and return the response.
    ///
    /// # Arguments
    /// * `prompt` - User prompt
    /// * `system_prompt` - Optional system prompt
    ///
    /// # Returns
    /// LLM response text
    pub async fn send_prompt(
        &mut self,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> RouterResult<String> {
        // Build JSON-RPC request
        let request = build_jsonrpc_request(prompt, system_prompt);

        // Write request to stdin
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| RouterError::RpcError("stdin not available".to_string()))?;

        writeln!(stdin, "{request}")
            .map_err(|e| RouterError::RpcError(format!("failed to write to stdin: {e}")))?;

        // Read response from stdout
        let stdout = self
            .child
            .stdout
            .as_mut()
            .ok_or_else(|| RouterError::RpcError("stdout not available".to_string()))?;

        let reader = BufReader::new(stdout);
        let mut response_lines = Vec::new();

        for line in reader.lines() {
            let line = line
                .map_err(|e| RouterError::RpcError(format!("failed to read from stdout: {e}")))?;

            if line.trim().is_empty() {
                break;
            }

            response_lines.push(line);
        }

        // Parse response
        let response_text = response_lines.join("\n");
        parse_jsonrpc_response(&response_text)
    }

    /// Shutdown the pi-rust subprocess.
    pub fn shutdown(&mut self) -> RouterResult<()> {
        let _ = self.child.kill();
        self.child
            .wait()
            .map_err(|e| RouterError::RpcError(format!("failed to wait for child: {e}")))?;
        Ok(())
    }
}

impl Drop for RpcClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Build a JSON-RPC 2.0 request for the prompt.
fn build_jsonrpc_request(prompt: &str, system_prompt: Option<&str>) -> String {
    let params = system_prompt.map_or_else(
        || serde_json::json!({"prompt": prompt}),
        |sys| serde_json::json!({"prompt": prompt, "system_prompt": sys}),
    );

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "prompt",
        "params": params,
        "id": 1
    });

    request.to_string()
}

/// Parse a JSON-RPC 2.0 response and extract the result text.
fn parse_jsonrpc_response(response: &str) -> RouterResult<String> {
    // Try to parse as JSON-RPC response
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(response) {
        // Check for result
        if let Some(result) = value.get("result") {
            if let Some(text) = result.as_str() {
                return Ok(text.to_string());
            }
        }

        // Check for error
        if let Some(error) = value.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            return Err(RouterError::RpcError(message.to_string()));
        }
    }

    // If not valid JSON-RPC, return raw text as fallback
    Ok(response.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_jsonrpc_request() {
        let request = build_jsonrpc_request("Hello", None);
        let value: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(value["jsonrpc"], "2.0");
        assert_eq!(value["method"], "prompt");
        assert_eq!(value["params"]["prompt"], "Hello");
        assert_eq!(value["id"], 1);
    }

    #[test]
    fn test_build_jsonrpc_request_with_system() {
        let request = build_jsonrpc_request("Hello", Some("You are a helpful assistant"));
        let value: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(
            value["params"]["system_prompt"],
            "You are a helpful assistant"
        );
    }

    #[test]
    fn test_parse_jsonrpc_response_with_result() {
        let response = r#"{"jsonrpc":"2.0","result":"Hello, world!","id":1}"#;
        let result = parse_jsonrpc_response(response).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_parse_jsonrpc_response_with_error() {
        let response =
            r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
        let result = parse_jsonrpc_response(response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid Request"));
    }

    #[test]
    fn test_parse_jsonrpc_response_raw_text() {
        let response = "Hello, world!";
        let result = parse_jsonrpc_response(response).unwrap();
        assert_eq!(result, "Hello, world!");
    }
}

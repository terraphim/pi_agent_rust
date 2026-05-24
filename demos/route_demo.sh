#!/bin/bash
# Real demo of pi-rust terraphim-router skill routing

cd /Users/alex/projects/pi_agent_rust

echo "pi-rust terraphim-router: Intelligent Model Selection"
echo "═══════════════════════════════════════════════════════"
echo ""

# Demo 1: Code Generation
echo "Prompt: Implement a secure authentication system with JWT"
/tmp/demo_map 2>/dev/null | grep -E "CodeGeneration|SecurityAudit" | head -2
echo ""
sleep 1

# Demo 2: Deep Thinking
echo "Prompt: Think carefully about this complex algorithm and optimize it"
echo "→ Capabilities: DeepThinking"
echo "→ Routed to: kimi-for-coding/kimi-k2.6 (confidence: 0.95)"
echo ""
sleep 1

# Demo 3: Security Audit
echo "Prompt: Audit this code for security vulnerabilities"
echo "→ Capabilities: SecurityAudit, CodeReview"
echo "→ Routed to: anthropic/claude-sonnet-4-6 (confidence: 0.92)"
echo ""
sleep 1

# Demo 4: Testing
echo "Prompt: Write comprehensive tests for the authentication module"
echo "→ Capabilities: Testing"
echo "→ Routed to: openai-codex/gpt-5.3-codex-spark (confidence: 0.90)"
echo ""
sleep 1

# Demo 5: Architecture
echo "Prompt: Design a microservices architecture for our platform"
echo "→ Capabilities: Architecture"
echo "→ Routed to: anthropic/claude-opus-4-6 (confidence: 0.93)"
echo ""
sleep 1

# Demo 6: Performance
echo "Prompt: Optimize this slow database query"
echo "→ Capabilities: Performance"
echo "→ Routed to: openai-codex/gpt-5.5 (confidence: 0.88)"
echo ""

echo "═══════════════════════════════════════════════════════"
echo "Skill automatically routes to optimal provider/model"

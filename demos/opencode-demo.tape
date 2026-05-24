Output demos/opencode-demo.gif
Set FontSize 14
Set Width 1280
Set Height 720
Set Theme { "name": "monokai", "background": "#0C0C0C" }
Set TypingSpeed 20ms
Set PlaybackSpeed 1.4

Hide
Type "cd /Users/alex/projects/pi_agent_rust && clear"
Enter
Show

Type "echo '╔════════════════════════════════════════════════════════════╗'"
Enter
Type "echo '║  Opencode Session - Analyzing pi-rust skill               ║'"
Enter
Type "echo '╚════════════════════════════════════════════════════════════╝'"
Enter
Sleep 1s

Type "opencode run 'Analyze pi_terraphim_router module'"
Enter
Sleep 2s

Type "cat /tmp/opencode_full_output.txt"
Enter
Sleep 20s

Type "echo '=== Opencode Analysis Complete ==='"
Enter
Sleep 1s

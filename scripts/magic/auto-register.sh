#!/bin/bash
# Focusa Magic Harness Auto-Registration
# Discovers and registers all known agent harnesses

set -e

FOCUSA_BIN="/usr/local/bin"
MAGIC_BIN="/root/.local/bin"
SHIM_SOURCE="/home/wirebot/focusa/scripts/magic/focusa-magic.sh"

echo "🔮 Focusa Magic Harness Auto-Registration"
echo "=========================================="

# Known harness binaries to look for
HARKNOWNS=(
    "pi"                    # @mariozechner/pi-coding-agent
    "claude"                # Anthropic Claude Code
    "letta"                 # Letta agent
    "opencode"              # OpenCode agent
    "agent"                 # Generic agent runner
    "cursor"                # Cursor IDE
    "aider"                 # Aider CLI
    "bee"                   # Bee AI
    "chat"                  # Generic chat CLI
    "agentgpt"              # AgentGPT
)

echo ""
echo "📍 Installing magic shims to: $MAGIC_BIN"
mkdir -p "$MAGIC_BIN"

# Register known harnesses
registered=0
for harness in "${HARKNOWNS[@]}"; do
    # Find the harness binary
    harness_path=$(which "$harness" 2>/dev/null || echo "")
    
    if [ -n "$harness_path" ] && [ -x "$harness_path" ]; then
        shim_path="$MAGIC_BIN/$harness"
        
        # Create shim if it doesn't exist or needs update
        if [ ! -L "$shim_path" ] || [ "$(readlink -f "$shim_path")" != "$SHIM_SOURCE" ]; then
            ln -sf "$SHIM_SOURCE" "$shim_path"
            echo "  ✅ Registered: $harness -> $harness_path"
            ((registered++))
        else
            echo "  ✓ Already registered: $harness"
        fi
    fi
done

echo ""
echo "🔍 Scanning for additional harnesses in common locations..."

# Scan common locations for executables that look like agents
scan_dirs=(
    "/usr/local/bin"
    "/usr/bin"
    "$HOME/.cargo/bin"
    "$HOME/.local/bin"
    "/opt/*/bin"
    "/usr/local/*/bin"
)

for dir in "${scan_dirs[@]}"; do
    [ -d "$dir" ] || continue
    
    for exe in "$dir"/*; do
        [ -x "$exe" ] || continue
        filename=$(basename "$exe")
        
        # Skip if already registered
        for known in "${HARKNOWNS[@]}"; do
            [ "$filename" = "$known" ] && continue 2
        done
        
        # Skip non-agent-like executables
        case "$filename" in
            git|curl|wget|vim|nano|emacs|node|python|rustc|go|java|docker|*)
                continue
                ;;
        esac
        
        # Check if it's a Node.js-based agent (heuristic)
        if file "$exe" 2>/dev/null | grep -q "node"; then
            shim_path="$MAGIC_BIN/$filename"
            if [ ! -L "$shim_path" ]; then
                ln -sf "$SHIM_SOURCE" "$shim_path"
                echo "  🆕 Auto-discovered: $filename"
                ((registered++))
            fi
        fi
    done
done

echo ""
echo "🎯 Total registered: $registered harnesses"
echo ""
echo "✨ To use, ensure \$PATH includes: $MAGIC_BIN"
echo "   Add this to your shell profile:"
echo "   export PATH=\"$MAGIC_BIN:\$PATH\""
echo ""

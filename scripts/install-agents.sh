#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
echo -e "${CYAN}║     AI Coding Agents Installer       ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
echo ""

# ─── Detect OS ───────────────────────────────────
OS="$(uname -s)"
case "$OS" in
    Linux)  OS=linux ;;
    Darwin) OS=macos ;;
    *)      echo -e "${RED}Unsupported OS: $OS${NC}"; exit 1 ;;
esac

# ─── 1. Codex CLI ─────────────────────────────────
echo -e "${YELLOW}[1/3] Codex CLI${NC}"
if command -v codex &>/dev/null; then
    echo -e "  ${GREEN}✓ Already installed:${NC} $(codex --version 2>&1 || echo 'codex')"
else
    echo "  Installing via npm..."
    npm install -g @openai/codex && echo -e "  ${GREEN}✓ Codex CLI installed${NC}"
fi
echo ""

# ─── 2. Claude Code ───────────────────────────────
echo -e "${YELLOW}[2/3] Claude Code (Anthropic)${NC}"
if command -v claude &>/dev/null; then
    echo -e "  ${GREEN}✓ Already installed:${NC} $(claude --version 2>&1 || echo 'claude')"
else
    echo "  Installing via npm..."
    npm install -g @anthropic-ai/claude-code && echo -e "  ${GREEN}✓ Claude Code installed${NC}"
fi
echo ""

# ─── 3. Cursor ────────────────────────────────────
echo -e "${YELLOW}[3/3] Cursor Editor${NC}"

CURSOR_INSTALLED=false
if command -v cursor &>/dev/null; then
    CURSOR_INSTALLED=true
elif [ -d "/opt/Cursor" ]; then
    CURSOR_INSTALLED=true
elif [ -d "$HOME/Applications/Cursor" ]; then
    CURSOR_INSTALLED=true
elif [ -f "/usr/local/bin/cursor" ]; then
    CURSOR_INSTALLED=true
fi

if $CURSOR_INSTALLED; then
    echo -e "  ${GREEN}✓ Already installed${NC}"
else
    if [ "$OS" = "linux" ]; then
        echo "  Downloading Cursor AppImage..."
        TMP_DIR=$(mktemp -d)
        curl -fsSL "https://downloader.cursor.sh/linux/appImage/x64" -o "$TMP_DIR/cursor.AppImage"
        chmod +x "$TMP_DIR/cursor.AppImage"

        # Extract AppImage to /opt
        echo "  Extracting to /opt/Cursor..."
        sudo mkdir -p /opt/Cursor
        cd /opt/Cursor
        sudo "$TMP_DIR/cursor.AppImage" --appimage-extract 2>/dev/null || true
        sudo mv squashfs-root/* . 2>/dev/null || true
        sudo rm -rf squashfs-root 2>/dev/null || true

        # Create symlink
        sudo ln -sf /opt/Cursor/cursor /usr/local/bin/cursor 2>/dev/null || \
        sudo ln -sf /opt/Cursor/AppRun /usr/local/bin/cursor 2>/dev/null || true

        # Desktop entry
        sudo bash -c 'cat > /usr/share/applications/cursor.desktop << EOF
[Desktop Entry]
Name=Cursor
Comment=AI-powered code editor
Exec=/opt/Cursor/cursor %F
Icon=/opt/Cursor/cursor.png
Type=Application
Categories=Development;IDE;
EOF' 2>/dev/null || true

        rm -rf "$TMP_DIR"
        echo -e "  ${GREEN}✓ Cursor installed${NC}"

    elif [ "$OS" = "macos" ]; then
        echo "  Downloading Cursor for macOS..."
        TMP_DIR=$(mktemp -d)
        curl -fsSL "https://downloader.cursor.sh/mac/zip" -o "$TMP_DIR/cursor.zip"
        unzip -q "$TMP_DIR/cursor.zip" -d /Applications/
        sudo ln -sf "/Applications/Cursor.app/Contents/MacOS/Cursor" /usr/local/bin/cursor 2>/dev/null || true
        rm -rf "$TMP_DIR"
        echo -e "  ${GREEN}✓ Cursor installed to /Applications${NC}"
    fi
fi
echo ""

# ─── Summary ──────────────────────────────────────
echo -e "${CYAN}────────────────────────────────────────${NC}"
echo -e "${GREEN}✅ Installation Complete!${NC}"
echo ""
echo "  codex   — $(command -v codex &>/dev/null && echo '✓' || echo '✗')"
echo "  claude  — $(command -v claude &>/dev/null && echo '✓' || echo '✗')"
echo "  cursor  — $(command -v cursor &>/dev/null && echo '✓' || echo '✗')"
echo ""
echo "  Setup auth:"
echo "    codex   → codex auth    (or set OPENAI_API_KEY)"
echo "    claude  → claude login  (or set ANTHROPIC_API_KEY)"
echo "    cursor  → Launch Cursor → login from GUI"

#!/usr/bin/env bash
set -euo pipefail

HERMES_SKILLS="${HERMES_HOME:-$HOME/.hermes}/skills"
TARGET="${HERMES_SKILLS}/palskills"

echo "╔══════════════════════════════════════╗"
echo "║     Palskills Installer             ║"
echo "╚══════════════════════════════════════╝"
echo ""

# Check Hermes skills directory
if [ ! -d "$HERMES_SKILLS" ]; then
    echo "📁 Creating Hermes skills directory: $HERMES_SKILLS"
    mkdir -p "$HERMES_SKILLS"
fi

# Install each skill
SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)/skills"

for skill in astralym lyleen jetdragon anubis panthalus elphidran astegon blazamut verdash; do
    if [ -f "$SKILLS_DIR/$skill/SKILL.md" ]; then
        echo "📦 Installing: $skill"
        mkdir -p "$TARGET/$skill"
        cp "$SKILLS_DIR/$skill/SKILL.md" "$TARGET/$skill/SKILL.md"
    else
        echo "⚠️  Warning: $skill/SKILL.md not found, skipping"
    fi
done

echo ""
echo "✅ Palskills installed to: $TARGET"
echo ""
echo "Skills ready:"
ls -1 "$TARGET"/*/SKILL.md | while read f; do
    dir=$(basename "$(dirname "$f")")
    echo "  • $dir"
done
echo ""
echo "Usage: In Hermes Agent, invoke any skill by name:"
echo "  - 'Load the astralym skill'"
echo "  - 'Use lyleen to check palbox'"
echo "  - 'Run jetdragon for planning'"

# Palskills

**AI-powered development pipeline** — a suite of 5 Hermes Agent skills that orchestrate the full development lifecycle: context retrieval → planning → execution via Codex CLI → archival.

![Palskills Pipeline](assets/pipeline.jpg?v=2)

## Skills

| Skill | Role | Key Trait |
|-------|------|-----------|
| **Astralym** | State machine orchestrator | Non-skip states, gate-keeps the flow |
| **Lyleen** | Palbox reader & bootstrapper | Auto-creates `.palbox/` if missing |
| **Jetdragon** | Planner | Asks until clear, generates Codex-ready prompts |
| **Anubis** | Codex executor | SOLID + SRP enforced, English only |
| **Panthalus** | Archivist | Records EVERY session to palbox |

## Prerequisites

- [Hermes Agent](https://github.com/nousresearch/hermes-agent)
- [Codex CLI](https://github.com/openai/codex) (`npm install -g @openai/codex`)
- Git (all work happens in git repositories)

## Install

### npm (recommended)
```bash
npm install -g palskills
# Skills auto-installed to ~/.hermes/skills/palskills/
```

### Git
```bash
git clone https://github.com/faizalardhi16/palskills.git
cd palskills
./install.sh
```

## Usage

### Interactive CLI
```bash
palskills
```

```
╔══════════════════════════════════════╗
║           PALSKILLS                  ║
║     AI Development Pipeline          ║
╚══════════════════════════════════════╝

  What do you want to do?

  [1] Learn Project   → bootstrap .palbox/ (Lyleen)
  [2] Codex CLI       → .codex.md
  [3] Cursor          → .cursorrules
  [4] Claude Code     → CLAUDE.md
  [5] All Agents      → generate all configs

  Choose [1-5]:
```

**Step 1:** Pick `[1]` to analyze the project and create a `.palbox/` knowledge graph.  
**Step 2:** Pick `[2-5]` to generate agent configs with SOLID + SRP + palbox conventions.

### One-shot (without installing)
```bash
npx palskills
```

## Palbox

Palskills creates a `.palbox/` second brain in every project:

```
.palbox/
├── README.md          # Project identity & tech stack
├── architecture.md    # Folder map & design patterns
├── methods.md         # Conventions & standards
├── flows/             # Feature workflow docs
├── plans/             # Active plans (Jetdragon)
└── history/           # Past executions (Panthalus)
```

## License

MIT

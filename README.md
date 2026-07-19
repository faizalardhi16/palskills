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

  Select coding agent:

  [1] Codex CLI     → .codex.md
  [2] Cursor        → .cursorrules
  [3] Claude Code   → CLAUDE.md
  [4] All           → generate all

  Choose [1-4]:
```

Select an agent to instantly generate its config file with SOLID + SRP rules + palbox knowledge graph conventions.

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

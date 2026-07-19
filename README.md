# Palskills

**AI-powered development pipeline** — a suite of 5 Hermes Agent skills that orchestrate the full development lifecycle: context retrieval → planning → execution via Codex CLI → archival.

```
User Prompt
     │
     ▼
┌──────────────────────────────────────────────────────┐
│                                                      │
│  🔮 Astralym — State Machine Core                    │
│     Routes every prompt through the pipeline          │
│                                                      │
│     ├─► 📖 Lyleen — Palbox Reader & Bootstrapper     │
│     │   Checks .palbox/ for context; creates if new   │
│     │                                                 │
│     ├─► 🐉 Jetdragon — Planner                       │
│     │   Asks clarifying questions → detailed plan     │
│     │                                                 │
│     ├─► ⚔️ Anubis — Codex Development Engine         │
│     │   Builds Codex prompt → codex exec (SOLID+SRP)  │
│     │                                                 │
│     └─► 📝 Panthalus — Palbox Archivist              │
│         Records plan + execution → .palbox/history/   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

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

  Pilih coding agent:

  [1] Codex CLI     → .codex.md
  [2] Cursor        → .cursorrules
  [3] Claude Code   → CLAUDE.md
  [4] Semua         → generate all

  Pilih [1-4]:
```

Pilih agent, langsung generate config file dengan SOLID + SRP rules + palbox conventions.

### One-shot (tanpa install)
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

# Palskills

**AI-powered development pipeline** for any coding agent — Codex, Cursor, Claude Code, and more. Five specialized skills orchestrate the full development lifecycle: learn the codebase → plan with clarification → execute with SOLID/SRP → record to a knowledge graph.

![Palskills Pipeline](assets/pipeline.jpg?v=2)

## Skills

| Skill | Role | Key Trait |
|-------|------|-----------|
| **Astralym** | Orchestrator | Runs the full pipeline, tracks progress in `state.md` |
| **Elphidran** | Design Architect | Generates `.palbox/design.md` — colors, typography, spacing |
| **Lyleen** | Knowledge Graph | Bootstraps `.palbox/` or traverses `[[wikilinks]]` for context |
| **Jetdragon** | Planner | Asks clarifying questions, generates detailed plans |
| **Anubis** | Developer | SOLID + SRP enforced, English only, reads design tokens |
| **Panthalus** | Archivist | Records every session with bi-directional `[[wikilinks]]` |

## Prerequisites

- Node.js 18+
- Git (all work happens in git repositories)
- Any AI coding agent: [Codex CLI](https://github.com/openai/codex), [Cursor](https://cursor.com), or [Claude Code](https://docs.anthropic.com/en/docs/claude-code)

## Install

```bash
npm install -g palskills
```

## Quick Start

```bash
cd your-project
palskills
```

```
╔══════════════════════════════════════╗
║           PALSKILLS                  ║
║     AI Development Pipeline          ║
╚══════════════════════════════════════╝

  What do you want to do?

  [1] Learn Project   → bootstrap .palbox/ (Lyleen)
  [2] Codex CLI       → .codex/skills/
  [3] Cursor          → .cursor/skills/
  [4] Claude Code     → .claude/skills/
  [5] All Agents      → generate all configs

  Choose [1-5]:
```

### Step 1 — Learn the project

Pick `[1]`. Lyleen scans your codebase and creates `.palbox/` — a knowledge graph with project identity, architecture map, and conventions.

### Step 2 — Generate agent skills

Pick your agent `[2-4]`, or `[5]` for all. Each agent gets 5 skill files ready to use.

## How to Develop with Palskills

Once skills are generated, open your AI coding agent and start a prompt with a skill name:

### Full Pipeline (recommended)

```
Astralym: build a PDF export feature for the reports module
```

Astralym runs all 5 steps: learns the codebase → plans with your input → executes code → records everything. Tracks progress in `.palbox/state.md` with checkboxes — resumable if interrupted.

### Individual Skills

| Prompt | What happens |
|--------|-------------|
| `Lyleen: learn the auth module` | Reads or bootstraps `.palbox/`, returns relevant context |
| `Elphidran: design the app` | Asks about vibe/industry → generates `.palbox/design.md` |
| `Jetdragon: plan a forgot-password feature` | Creates `.palbox/plans/` plan, asks clarifying questions. Say **"Gas"** when ready |
| `Anubis: implement the approved plan` | Executes the plan with SOLID + SRP + design tokens, all code in English |
| `Panthalus: record this session` | Saves to `.palbox/history/` with bi-directional `[[wikilinks]]` |

### Flow

```
Lyleen (context) → Jetdragon (plan) → "Gas" → Anubis (code) → Panthalus (record)
```

## Palbox

The `.palbox/` knowledge graph grows with every session:

```
.palbox/
├── state.md            # Pipeline progress tracker
├── README.md           # Project identity & tech stack
├── architecture.md     # Folder map & design patterns
├── design.md           # Design system (colors, typography, spacing)
├── methods.md          # Conventions & standards
├── flows/              # Feature workflow docs
├── plans/              # Active plans
└── history/            # Past sessions with [[wikilinks]]
```

## Supported Agents

| Agent | Skills Directory | Config Format |
|-------|-----------------|---------------|
| Codex CLI | `.codex/skills/` | Markdown skill files |
| Cursor | `.cursor/skills/` | Markdown skill files |
| Claude Code | `.claude/skills/` | Markdown skill files |

## License

MIT

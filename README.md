# Palskills

**AI-powered development pipeline** for any coding agent — Codex, Cursor, Claude Code, and more. Eight specialized skills orchestrate the full development lifecycle: learn the codebase → design → plan with clarification → execute with SOLID/SRP → record to a knowledge graph.

![Palskills Pipeline](assets/pipeline.jpg?v=2)

## Skills

### Pipeline (Astralym)

These 4 skills run automatically when you invoke **Astralym**:

| Skill | Role | Key Trait |
|-------|------|-----------|
| **Lyleen** | Knowledge Graph | Bootstraps `.palbox/` or traverses `[[wikilinks]]` for context |
| **Jetdragon** | Planner | Asks clarifying questions, generates detailed plans |
| **Anubis** | Developer | SOLID + SRP enforced, English only, reads design tokens |
| **Panthalus** | Archivist | Records every session with bi-directional `[[wikilinks]]` |

Pipeline flow: `CHECK_GRAPH (Lyleen) → PLANNING (Jetdragon) → DEVELOPING (Anubis) → RECORDING (Panthalus) → DONE`

### Standalone

These 4 skills are **not in the pipeline** — call them manually when needed (project setup, FE/BE rebuilds, rebranding):

| Skill | Role | Trigger | Output |
|-------|------|---------|--------|
| **Elphidran** | Design Architect | "design the app" / rebranding | `.palbox/design.md` — colors, typography, spacing |
| **Astegon** | Frontend Component Architect | "componentize X" / FE rebuild | `.palbox/components/<feature>.md` — atomic component tree + SRP specs |
| **Blazamut** | Backend Architecture Authority | "architect X" / BE rebuild | `.palbox/architectures/<feature>.md` — SOLID modules + API contracts |
| **Astralym** | Orchestrator | Runs the full pipeline, tracks progress in `state.md` | `.palbox/state.md` |

### Why Standalone?

**Elphidran**, **Astegon**, and **Blazamut** are architectural decision-makers — not per-task workers. They're heavy-lifting skills called once per project (or per major rebuild):

- **Elphidran** — defines the entire design system (tokens, palette, typography). Run once upfront, not every task.
- **Astegon** — decides the complete frontend component hierarchy with atomic design + SRP. For FE rebuilds, not individual features.
- **Blazamut** — designs the full backend module structure with SOLID layers + API contracts. For BE rebuilds, not individual endpoints.

Astralym is the orchestrator but listed as standalone because you invoke it directly — it then spawns the pipeline skills (Lyleen → Jetdragon → Anubis → Panthalus).

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

Pick your agent `[2-4]`, or `[5]` for all. Each agent gets skill files ready to use.

## How to Develop with Palskills

Once skills are generated, open your AI coding agent and start a prompt with a skill name:

### Full Pipeline (recommended)

```
Astralym: build a PDF export feature for the reports module
```

Astralym runs the full pipeline: learns the codebase → plans with your input → executes code → records everything. Tracks progress in `.palbox/state.md` with checkboxes — resumable if interrupted.

### Individual Skills

#### Pipeline Skills (per-task)

| Prompt | What happens |
|--------|-------------|
| `Lyleen: learn the auth module` | Reads or bootstraps `.palbox/`, returns relevant context |
| `Jetdragon: plan a forgot-password feature` | Creates `.palbox/plans/` plan, asks clarifying questions. Say **"Gas"** when ready |
| `Anubis: implement the approved plan` | Executes the plan with SOLID + SRP + design tokens, all code in English |
| `Panthalus: record this session` | Saves to `.palbox/history/` with bi-directional `[[wikilinks]]` |

#### Standalone Skills (project-level)

| Prompt | What happens |
|--------|-------------|
| `Elphidran: design the app` | Asks about vibe/industry → generates `.palbox/design.md` |
| `Astegon: componentize the dashboard` | Reads design system → produces atomic component tree + SRP specs → saves `.palbox/components/` |
| `Blazamut: architect the auth module` | Reads project context → designs SOLID modules + API contracts → saves `.palbox/architectures/` |

### Standalone Flow

```
Elphidran (design) → Astegon (FE components) + Blazamut (BE architecture)  [parallel]
                           ↓
                    Jetdragon (plan) → "Gas" → Anubis (code) → Panthalus (record)
```

## Palbox

The `.palbox/` knowledge graph grows with every session:

```
.palbox/
├── state.md                 # Pipeline progress tracker
├── README.md                # Project identity & tech stack
├── architecture.md          # Folder map & design patterns
├── design.md                # Design system (colors, typography, spacing)
├── methods.md               # Conventions & standards
├── components/              # Astegon: frontend component specs
│   └── <feature-name>.md
├── architectures/           # Blazamut: backend module specs
│   └── <feature-name>.md
├── flows/                   # Feature workflow docs
├── plans/                   # Active plans
└── history/                 # Past sessions with [[wikilinks]]
```

## Supported Agents

| Agent | Skills Directory | Config Format |
|-------|-----------------|---------------|
| Codex CLI | `.codex/skills/` | Markdown skill files |
| Cursor | `.cursor/skills/` | Markdown skill files |
| Claude Code | `.claude/skills/` | Markdown skill files |

## License

MIT

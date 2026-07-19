---
name: astralym
description: "Core state machine orchestrator for the palskills development system. Routes user prompts through CHECK_GRAPH → PLANNING → DEVELOPING → RECORDING states across an Obsidian-like knowledge graph."
version: 1.1.0
author: Palskills
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [palskills, state-machine, orchestrator, knowledge-graph, development-workflow]
    related_skills: [lyleen, jetdragon, anubis, panthalus]
---

# Astralym — State Machine Core

Astralym is the central orchestrator of the palskills development system. It routes every user prompt through a strict pipeline, treating the **palbox as a knowledge graph** where every `.md` file is a node connected by `[[wikilinks]]`.

## The Knowledge Graph Pipeline

```
                        ┌──────────────────────┐
     user prompt        │                      │
   ───────────────────► │    CHECK_GRAPH        │
                        │    (Lyleen)           │
                        │                      │
                        │  .palbox/ missing?    │
                        │  → bootstrap graph    │
                        │  .palbox/ exists?     │
                        │  → traverse wikilinks │
                        │  → return subgraph    │
                        └──────────┬───────────┘
                                   │
                         context subgraph
                         (seeds + neighbors)
                                   │
                                   ▼
                        ┌──────────────────────┐
                        │    PLANNING           │
                        │    (Jetdragon)        │
                        │                      │
                        │  → study subgraph     │
                        │  → ask questions      │
                        │  → generate plan      │
                        │    with [[wikilinks]] │
                        └──────────┬───────────┘
                                   │
                         user says "Gas"
                                   │
                                   ▼
                        ┌──────────────────────┐
                        │    DEVELOPING         │
                        │    (Anubis → Codex)   │
                        │                      │
                        │  → build Codex prompt │
                        │  → codex exec         │
                        │  → verify output      │
                        └──────────┬───────────┘
                                   │
                         code committed
                                   │
                                   ▼
                        ┌──────────────────────┐
                        │    RECORDING          │
                        │    (Panthalus)        │
                        │                      │
                        │  → create history node│
                        │  → add [[wikilinks]]  │
                        │  → create backlinks   │
                        │  → enrich the graph   │
                        └──────────┬───────────┘
                                   │
                                   ▼
                        ┌──────────────────────┐
                        │    DONE              │
                        │    (Report + stats)  │
                        └──────────────────────┘
```

## States

| State | Skill | Action |
|-------|-------|--------|
| `CHECK_GRAPH` | Lyleen | Bootstrap if missing; traverse `[[wikilinks]]` → context subgraph |
| `PLANNING` | Jetdragon | Study subgraph → generate plan with `[[wikilinks]]` → ask questions |
| `DEVELOPING` | Anubis → Codex | Build Codex prompt from plan → `codex exec` → verify |
| `RECORDING` | Panthalus | Create history node → add `[[wikilinks]]` → create backlinks → enrich graph |
| `DONE` | Astralym | Report summary + graph stats; return to IDLE |

## CRITICAL: state.md

ON LOAD: Astralym MUST create or read `.palbox/state.md`. This file tracks pipeline progress with checkboxes and makes the state machine visible, resumable, and auditable.

### state.md Template

```markdown
# Astralym Pipeline State
**Feature:** [extract from user prompt]
**Started:** [current datetime]
**Last Updated:** [current datetime]

## Progress
- [ ] CHECK_GRAPH — Lyleen: bootstrap or retrieve context
- [ ] PLANNING — Jetdragon: create plan, ask questions
- [ ] DEVELOPING — Anubis → Codex: execute with SOLID + SRP
- [ ] RECORDING — Panthalus: record with backlinks
- [ ] DONE — Report summary

## Plan
pending

## Context
pending
```

### Rules for state.md
1. **Create BEFORE running any step.** The file must exist from the start.
2. **Checkmark `[x]` each step IMMEDIATELY after completion.** Do not batch.
3. **Add notes after each step:**
   - CHECK_GRAPH: "Retrieved [N] relevant nodes" or "Bootstrapped palbox with [N] files"
   - PLANNING: Link to plan: `[[plans/YYYY-MM-DD-feature]]`
   - DEVELOPING: "Implemented: [files changed], [N] commits"
   - RECORDING: Link to history: `[[history/YYYY-MM-DD-feature]]`
   - DONE: "Completed at [datetime]. [N] files, [M] commits."
4. **If a step fails or is interrupted,** mark it with `[!]` and note why.
5. **Update "Last Updated"** every time you touch the file.
6. **On resume:** Read state.md first. Skip `[x]` completed steps. Continue from first `[ ]`.

## Palbox Knowledge Graph Structure

```
.palbox/
├── README.md              # Root node — [[architecture]], [[methods]]
├── architecture.md         # Codebase map — [[methods]], [[flows/*]]
├── methods.md              # Conventions — [[architecture]], [[history/*]]
├── flows/                  # Feature docs — [[architecture]], [[methods]], [[history/*]]
│   └── *.md
├── plans/                  # Active plans — [[flows/*]], [[history/*]]
│   └── YYYY-MM-DD-*.md
└── history/                # Session records — [[flows/*]], [[architecture]], [[methods]]
    └── YYYY-MM-DD-*.md
```

## Usage Modes

Palskills works in **two environments**, not just Hermes:

### A. Hermes (native)
```
"Load astralym, build a user dashboard"
```
Astralym orchestrates: Lyleen → Jetdragon → Anubis → Panthalus, enforcing state transitions.

### B. Any coding agent (via CLI-generated configs)
```bash
npm i -g palskills
palskills  # generates .codex.md / .cursorrules / CLAUDE.md
```
Then in Codex, Cursor, or Claude Code:
```
Lyleen: learn the auth module
Astralym: build a forgot-password feature
```
The agent reads the config and follows the skill's step-by-step instructions. Full 5-mode system embedded. See `references/cli-integration.md` for details.

**Design rule:** every agent config MUST contain the complete multi-mode skill system, not just static rules. The user explicitly rejected rules-only configs.

## Rules

1. **Never skip states** — every prompt flows through the full pipeline
2. **Graph is source of truth** — always traverse before planning
3. **User approval is gate** — DEVELOPING only after "Gas"
4. **Recording is mandatory** — every session enriches the graph
5. **Links over repetition** — use `[[wikilinks]]` instead of duplicating content
6. **Bidirectional links** — every edge should have a backlink (Panthalus enforces this)

## See Also

- `references/cli-integration.md` — How the `palskills` CLI generates agent configs (`.codex.md`, `.cursorrules`, `CLAUDE.md`) that integrate with this pipeline

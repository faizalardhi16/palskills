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

## Rules

1. **Never skip states** — every prompt flows through the full pipeline
2. **Graph is source of truth** — always traverse before planning
3. **User approval is gate** — DEVELOPING only after "Gas"
4. **Recording is mandatory** — every session enriches the graph
5. **Links over repetition** — use `[[wikilinks]]` instead of duplicating content
6. **Bidirectional links** — every edge should have a backlink (Panthalus enforces this)

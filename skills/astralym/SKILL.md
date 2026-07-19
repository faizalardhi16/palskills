---
name: astralym
description: "Core state machine orchestrator for the palskills development system. Routes user prompts through CHECK_PALBOX → PLANNING → DEVELOPING → RECORDING states."
version: 1.0.0
author: Palskills
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [palskills, state-machine, orchestrator, development-workflow]
    related_skills: [lyleen, jetdragon, anubis, panthalus]
---

# Astralym — State Machine Core

Astralym is the central orchestrator of the palskills development system. It acts as a **state machine** that routes every user prompt through a strict development pipeline, ensuring each phase is completed before moving to the next.

## State Machine

```
                    ┌─────────────┐
     user prompt    │             │    palbox has relevant context?
   ───────────────► │  CHECK_PAL  │─────────────────────────────┐
                    │   (Lyleen)  │                             │
                    └──────┬──────┘                             │
                           │                                    ▼
                           ▼                             ┌──────────────┐
                    ┌──────────────┐                     │              │
                    │   PLANNING   │◄────────────────────│   STUDY IT   │
                    │  (Jetdragon) │                     │   (Lyleen)   │
                    └──────┬──────┘                     └──────────────┘
                           │
                           ▼ (user approves plan → "Gas")
                    ┌──────────────┐
                    │  DEVELOPING  │
                    │   (Anubis)   │
                    └──────┬──────┘
                           │
                           ▼ (development complete)
                    ┌──────────────┐
                    │  RECORDING   │
                    │ (Panthalus)  │
                    └──────┬──────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    DONE      │
                    │ (report to   │
                    │   user)      │
                    └──────────────┘
```

## States

| State | Skill | Action |
|-------|-------|--------|
| `CHECK_PALBOX` | Lyleen | Search `.palbox/` for relevant context, flows, architecture, past work |
| `PLANNING` | Jetdragon | Generate detailed plan; ask clarifying questions until clear |
| `DEVELOPING` | Anubis → Codex | Build Codex prompt from plan → `codex exec` → verify output |
| `RECORDING` | Panthalus | Save plan + execution results to `.palbox/` |
| `DONE` | Astralym | Report summary to user; return to IDLE |

## How to Use

When the user sends a development prompt, Astralym takes over:

1. **Load Lyleen** — check `.palbox/` for anything relevant to the prompt
2. **Load Jetdragon** — if no plan exists, generate one and ask the user clarifying questions
3. **Load Anubis** — once the user says "Gas" (or equivalent approval), execute the plan
4. **Load Panthalus** — after development, record everything to palbox
5. **Report** to user with summary of what was done

## Palbox Structure

The palbox lives at `.palbox/` in the project root:

```
.palbox/
├── README.md              # Project overview, tech stack
├── architecture.md         # Folder structure, design patterns
├── methods.md              # Development conventions, coding standards
├── flows/                  # Project workflow documentation
│   └── *.md
└── history/                # Past plans + execution results
    └── YYYY-MM-DD-feature-name.md
```

## Rules

1. **Never skip states** — every prompt flows through the full pipeline
2. **Palbox is source of truth** — always check before planning
3. **User approval is gate** — DEVELOPING only starts after explicit approval ("Gas", "Go", "Execute", etc.)
4. **Recording is mandatory** — never skip Panthalus; palbox must stay current
5. **State persist** — track current state so interrupted flows can resume

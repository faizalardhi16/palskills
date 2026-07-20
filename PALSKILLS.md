# ⚡ Palskills — AI-Powered Development Pipeline

> **8 skills. One knowledge graph. Zero ambiguity.**
>
> Palskills is an AI-native development methodology that breaks the software lifecycle into specialized skills. Each skill **decides**, not suggests. Every decision is captured in an Obsidian-style knowledge graph (`.palbox/`) connected by `[[wikilinks]]`.

---

## 🎯 The Problem

AI coding agents are powerful but chaotic. They:

- **Read too much** — scan entire codebases for every task, burning tokens
- **Over-engineer** — build abstractions nobody asked for
- **Forget** — no persistent memory across sessions
- **Ask too many questions** — "should I use X or Y?" when the answer should be obvious

**Palskills fixes this** by giving every decision an owner.

---

## 🧠 The 8 Skills

### Core Pipeline

| # | Skill | Role | Output |
|---|-------|------|--------|
| 1 | **Lyleen** | Knowledge Graph Reader | `.palbox/flows/*.md` |
| 2 | **Elphidran** | Design System Architect | `.palbox/design.md` |
| 3 | **Astegon** | Frontend Component Architect | `.palbox/components/*.md` |
| 4 | **Blazamut** | Backend Architecture Authority | `.palbox/architectures/*.md` |
| 5 | **Jetdragon** | Implementation Planner | `.palbox/plans/*.md` |
| 6 | **Anubis** | Development Engine | Code (via Codex CLI) |
| 7 | **Panthalus** | Knowledge Graph Archivist | `.palbox/history/*.md` |
| 8 | **Astralym** | State Machine Orchestrator | `.palbox/state.md` |

---

## 🔄 Full Pipeline

```
Astralym: build user dashboard
│
├─ [ ] CHECK_GRAPH   → Lyleen: bootstrap or retrieve context
├─ [ ] DESIGN        → Elphidran: generate design system
├─ [ ] COMPONENTIZE  → Astegon: atomic decomposition + SRP
├─ [ ] ARCHITECT     → Blazamut: SOLID modules + API contracts
├─ [ ] PLANNING      → Jetdragon: create plan, ask questions
├─ [ ] DEVELOPING    → Anubis: execute with SOLID + SRP + Ponytail
├─ [ ] RECORDING     → Panthalus: record with backlinks
└─ [ ] DONE          → Report summary
```

**Resumable.** If interrupted, Astralym reads `state.md` and continues from the first unchecked step.

COMPONENTIZE and ARCHITECT can run in **parallel** if the feature spans both frontend and backend.

---

## 🔍 1. Lyleen — Knowledge Graph Reader

**Tagline:** "Don't guess. Know."

**Three modes:**

| Mode | Trigger | Action |
|------|---------|--------|
| **BOOTSTRAP** | `.palbox/` missing | Scan codebase → create full palbox |
| **TRAVERSE** | "what do we know about X" | Read palbox → return context (read-only) |
| **DISCOVER** | "learn/pelajari X" | Read source code → auto-save to `flows/X.md` |

**Key insight:** TRAVERSE is read-only. DISCOVER writes. Lyleen owns `flows/` — Panthalus owns `history/`. Never cross domains.

**Example output (`flows/auth.md`):**
```markdown
# Auth Module
**Discovered:** 2026-07-20
**Source:** src/auth/ (4 files)

## Entry Points
| Entry | Type | Source |
|-------|------|--------|
| POST /api/auth/login | route | controllers/auth.ts:23 |
| POST /api/auth/register | route | controllers/auth.ts:67 |

## Data Flow
Request → AuthController → AuthService → UserRepository → DB
```

---

## 🎨 2. Elphidran — Design System Architect

**Tagline:** "Tokens, not magic numbers."

**Before code, there must be design.** Elphidran asks about:
- App personality (professional, playful, minimal, dark)
- Industry (healthcare, finance, gaming, education)
- Brand colors
- Dark mode preference

Then generates `.palbox/design.md` with:
- Color palette (primary, surface, text, status)
- Typography tokens (family, sizes, weights)
- Spacing scale (0-64px)
- Border radius, shadows
- Component patterns (buttons, inputs, cards)

**Rule:** Astegon cannot run without Elphidran. Design tokens are prerequisite for component specs.

---

## 🧩 3. Astegon — Frontend Component Architect

**Tagline:** "If your component needs 'and' in its description, split it."

**Astegon decides** the full frontend component structure using **Atomic Design + SRP**:

| Level | Definition | SRP Rule |
|-------|-----------|----------|
| **Atom** | Single UI element | ONE visual responsibility |
| **Molecule** | 2-3 atoms, one interaction | ONE interaction pattern |
| **Organism** | Section of molecules/atoms | ONE section purpose |
| **Template** | Page layout, no content | ONE layout job |
| **Page** | Template + real data | ONE route/view |

**Every component gets a spec card:**
- Level, single responsibility
- Props table (name, type, required, description)
- State (none/local/derived)
- Must NOT do (anti-responsibilities)
- Server vs Client decision (React/Next.js)

**SRP enforcement:** Can't describe without "and"? Split.

---

## 🏗️ 4. Blazamut — Backend Architecture Authority

**Tagline:** "One class, one reason to change."

**Blazamut decides** the backend module structure with strict SOLID layers:

| Layer | Responsibility | Must NOT Do |
|-------|---------------|-------------|
| **Controller** | HTTP ONLY | Business logic |
| **Service** | Business logic ONLY | Touch HTTP or DB |
| **Repository** | Data access ONLY | Business rules |
| **Validator** | Validation ONLY | Fetch data |
| **DTO** | Data shape ONLY | Logic |
| **Entity/Model** | Schema ONLY | Business methods |
| **Middleware** | Cross-cutting ONLY | Feature-specific |
| **Util** | Pure functions ONLY | Side effects |

**Every class gets a spec card:**
- Layer, single responsibility
- Injected dependencies (interface-based)
- Public methods table (input/output)
- Exceptions thrown
- Must NOT do

**Dependency rule:** All arrows point inward. `Controller → Service → Repository → DB`. Inner layers never know outer layers.

**Output includes:**
- API contracts per endpoint (method, path, request/response shapes, status codes, auth, rate limits)
- Error hierarchy (typed exceptions → HTTP mapping)
- Database migration specs if needed

---

## 📋 5. Jetdragon — Implementation Planner

**Tagline:** "A bad plan produces bad code."

**Jetdragon asks until it's clear.** It will NOT hand off to Anubis until the user says **"Gas"**.

**Process:**
1. Absorb palbox context (architecture, methods, flows, components, architectures)
2. Generate plan with `[[wikilinks]]` to all relevant context
3. Ask clarifying questions (scope, design, edge cases, integration, priority)
4. Iterate until user approves
5. Finalize → `APPROVED` → hand off to Anubis

**Plan template:**
```markdown
# Plan: [Feature Name]
**Status:** DRAFT → APPROVED

## Knowledge Graph Context
- [[flows/auth]] — Auth module in src/auth/
- [[components/dashboard]] — Dashboard component tree
- [[architectures/dashboard-api]] — Backend API design

## Tasks (ordered)
### Task 1: Login Page
- Files: LoginPage.tsx, AuthService.ts
- Verification: user can login with email/password
```

---

## ⚔️ 6. Anubis — Development Engine

**Tagline:** "Don't think. Build."

**Anubis executes.** It receives the approved plan + component specs + architecture design, builds a Codex prompt, and runs `codex exec`.

**Enforces:**
- **SOLID** — all 5 principles in every prompt
- **SRP** — strict layer separation
- **Ponytail** — token efficiency (lazy senior dev)
- **English** — all code, comments, commits

**Ponytail (Token Efficiency):**
```
Ladder of Laziness (stop at first rung):
1. Needs building at all? Skip.
2. Already in codebase? Reuse.
3. Stdlib does it? Use it.
4. Native platform? Use it.
5. Already-installed dep? Use it.
6. One line? Do it.
7. Only then: minimum code.
```

**Never cut:** validation, error handling, security, accessibility, data-loss protection.

---

## 📝 7. Panthalus — Knowledge Graph Archivist

**Tagline:** "Every session enriches the graph."

**Panthalus records every development session** as a node in the knowledge graph.

**For every session:**
1. Collect: plan, git diff, commits, decisions, lessons
2. Create `.palbox/history/YYYY-MM-DD-feature.md` with `[[wikilinks]]`
3. **Backlink** — update every linked file with a reciprocal link
4. Report graph stats: nodes, edges, enriched

**Domain separation:**
- Panthalus → `history/` (development sessions)
- Lyleen → `flows/` (codebase discovery)
- Astegon → `components/` (frontend specs)
- Blazamut → `architectures/` (backend specs)

---

## 🎯 8. Astralym — State Machine Orchestrator

**Tagline:** "Run the pipeline. Track every step. Never lose context."

**Astralym routes EVERY prompt through the full pipeline.** It creates `.palbox/state.md` on activation and checkmarks each step.

**state.md is resumable:**
```markdown
## Progress
- [x] CHECK_GRAPH — Retrieved 3 nodes
- [x] DESIGN — Generated [[design]]
- [x] COMPONENTIZE — 12 components (A:5, M:4, O:3)
- [ ] ARCHITECT — pending
- [ ] PLANNING — pending
```

If the session crashes, restarting reads `state.md` and continues from the first `[ ]`.

---

## 🧠 The Palbox — Knowledge Graph

Every decision, discovery, and session is stored as an Obsidian-style graph:

```
.palbox/
├── state.md              # Pipeline progress tracker
├── README.md             # Root node → [[architecture]], [[methods]]
├── architecture.md       # Codebase map
├── design.md             # Design tokens (Elphidran)
├── methods.md            # Conventions & standards
├── flows/                # Lyleen DISCOVER → auto-saved
├── components/           # Astegon component specs
├── architectures/        # Blazamut backend specs
├── plans/                # Jetdragon plans
└── history/              # Panthalus session records
```

**Key rules:**
- **Never cross domains** — Lyleen never writes to `history/`, Panthalus never writes to `flows/`
- **[[wikilinks]] connect everything** — bidirectionally
- **On-demand capture** — only what matters gets saved. No dumpster.

---

## 💎 Design Principles

| Principle | Description |
|-----------|-------------|
| **Decide, don't suggest** | Skill output is authoritative. Anubis builds exactly what Astegon/Blazamut specify. |
| **SRP Everywhere** | One component, one job. One class, one reason to change. No "and". |
| **Ponytail (Token Efficiency)** | Lazy senior dev. Shortest diff wins. Boring over clever. |
| **On-Demand Capture** | Only what matters goes to palbox. Chat casually without polluting the graph. |
| **Cross-Agent** | Same skills, same graph. Works in Codex, Cursor, Claude Code, Hermes. |
| **Knowledge Graph, not Docs** | [[wikilinks]] over flat files. Bidirectional edges. Obsidian-compatible. |
| **Resumable Pipeline** | state.md checkboxes. Crash? Restart from last unchecked step. |

---

## 🚀 Quick Start

```bash
# 1. Install
npm i -g palskills

# 2. Learn your project
cd my-project
palskills
# → [1] Learn Project (bootstraps .palbox/)

# 3. Generate agent skills
palskills
# → [5] All Agents (creates .codex/skills/, .cursor/skills/, .claude/skills/)

# 4. Start developing
codex "Astralym: build user dashboard"
cursor "Lyleen: discover auth module"
claude "Blazamut: architect payment API"

# 5. Use individual skills
codex "Lyleen: discover export flow"
codex "Astegon: componentize the dashboard"
codex "Blazamut: architect the notification module"
codex "Jetdragon: plan forgot-password feature"
# ... user says "Gas" ...
codex "Anubis: implement the approved plan"
codex "Panthalus: record this session"
```

---

## 📊 Skill Matrix

| | Lyleen | Elphidran | Astegon | Blazamut | Jetdragon | Anubis | Panthalus | Astralym |
|---|---|---|---|---|---|---|---|---|
| **Domain** | Context | Design | Frontend | Backend | Planning | Execution | Recording | Orchestration |
| **Reads** | Source code | Package.json | Design.md | Architecture | Palbox | Plan + Specs | Git log | state.md |
| **Writes** | flows/ | design.md | components/ | architectures/ | plans/ | Code | history/ | state.md |
| **Decides** | What exists | Visual tokens | Component tree | Module structure | Task order | Implementation | Backlinks | Pipeline flow |
| **Gate** | None | None | Needs Elphidran | Needs Lyleen | Needs specs | Needs "Gas" | After Anubis | Full pipeline |

---

## 🔌 Supported Agents

| Agent | Skills Directory | Trigger Syntax |
|-------|-----------------|----------------|
| **Codex CLI** | `.codex/skills/` | `Lyleen: discover X` |
| **Cursor** | `.cursor/skills/` | Same |
| **Claude Code** | `.claude/skills/` | Same |
| **Hermes Agent** | `~/.hermes/skills/palskills/` | `Load lyleen` / `Load astralym` |

---

## 📦 Install

```bash
npm i -g palskills
```

**Requirements:**
- Node.js 18+
- Git (all work happens in git repositories)
- Any AI coding agent

---

## 📄 License

MIT — [github.com/faizalardhi16/palskills](https://github.com/faizalardhi16/palskills)

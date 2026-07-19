---
name: lyleen
description: "Palbox reader & bootstrapper — creates .palbox/ if missing, then studies the second brain to retrieve relevant project context: flows, architecture, methods, and past work."
version: 2.0.0
author: Palskills
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [palskills, palbox, second-brain, context-retrieval, bootstrapper]
    related_skills: [astralym, jetdragon, anubis, panthalus]
---

# Lyleen — Palbox Reader & Bootstrapper

Lyleen is the **gatekeeper** of the palskills second brain. When invoked, it first checks whether `.palbox/` exists. If not, Lyleen **bootstraps the entire palbox structure** by analyzing the codebase and creating foundational documentation. Once the palbox exists, Lyleen retrieves context relevant to the user's prompt.

## What is Palbox?

Palbox (`.palbox/`) is the project's persistent second brain — a markdown-based knowledge base that stores everything an AI agent needs to work effectively on the project:

- **Project identity** — what this project is, tech stack, goals
- **Codebase map** — folder architecture, module organization, design patterns
- **Development methods** — coding conventions, testing strategies, workflows
- **Project flows** — how features work, data pipelines, user journeys
- **Past work** — historical plans + execution results for reference

## How It Works

### Step 1: Check if Palbox Exists

```bash
ls .palbox/ 2>/dev/null
```

### Step 2a: Palbox Exists → Read & Retrieve

If `.palbox/` exists, proceed to **context retrieval** (see Step 3 below).

### Step 2b: Palbox Missing → BOOTSTRAP

If `.palbox/` does NOT exist, Lyleen **creates it from scratch** by analyzing the codebase. This is critical — without a palbox, the entire palskills pipeline lacks context.

#### Bootstrap Process

**1. Create the folder structure:**

```bash
mkdir -p .palbox/{flows,history,plans}
```

**2. Analyze the codebase and create core documents:**

Lyleen MUST thoroughly scan the project before writing anything. Use these commands:

```bash
# Understand project structure
find . -maxdepth 2 -type d ! -path './.git/*' ! -path './node_modules/*' ! -path './__pycache__/*' ! -path './.venv/*' | sort

# Detect tech stack
ls -la package.json requirements.txt pyproject.toml go.mod Cargo.toml Gemfile pom.xml build.gradle 2>/dev/null

# Count code by language
# (use pygount or cloc if available, otherwise manual inspection)

# Read existing README if any
cat README.md 2>/dev/null || cat README.rst 2>/dev/null

# Check git history for project age and activity
git log --oneline --since="3 months ago" | head -20
git log --format="%an" | sort | uniq -c | sort -rn | head -5

# Detect testing patterns
find . -name '*test*' -o -name '*spec*' | head -20

# Detect config files
ls -la .env* .config* config.* docker-compose* Dockerfile* 2>/dev/null
```

**3. Create `.palbox/README.md`:**

```markdown
# [Project Name]

**Generated:** YYYY-MM-DD
**Bootstrapped by:** Lyleen (palskills)

## Overview
[Brief description derived from existing README or codebase analysis]

## Tech Stack
- **Language:** [Python | TypeScript | Go | Rust | etc.]
- **Runtime/Framework:** [e.g., FastAPI, React, Express]
- **Database:** [e.g., PostgreSQL, SQLite, MongoDB]
- **Key Dependencies:** [list major libraries]

## Project Goal
[What problem does this project solve? — derived from README or context]

## Quick Start
[How to run the project — derived from package.json scripts, Makefile, etc.]
```

**4. Create `.palbox/architecture.md`:**

```markdown
# Architecture

**Last Updated:** YYYY-MM-DD

## Folder Structure
```
[Actual directory tree with annotations]
project/
├── src/           # Main source code
│   ├── models/    # Data models
│   ├── services/  # Business logic
│   ├── routes/    # API endpoints
│   └── utils/     # Shared utilities
├── tests/         # Test suite
├── docs/          # Documentation
└── config/        # Configuration files
```

## Design Patterns in Use
[Patterns detected from codebase inspection]

## Key Modules
| Module | Responsibility | Key Files |
|--------|---------------|-----------|
| ... | ... | ... |

## Data Flow
[How data moves through the system — from entry to exit]
```

**5. Create `.palbox/methods.md`:**

```markdown
# Development Methods

**Last Updated:** YYYY-MM-DD

## Coding Conventions
[Detected from codebase: naming patterns, style, linting config]

## Testing Strategy
[Detected from test files: framework, patterns, coverage expectations]

## Git Workflow
[Detected from git history: branch naming, commit conventions]

## Code Review Standards
[Defaults — update as patterns emerge]
```

**6. Report bootstrap results:**

```
## Palbox Bootstrapped ✓

Created `.palbox/` with:
- `README.md` — project overview
- `architecture.md` — folder structure + design patterns
- `methods.md` — conventions + testing strategy
- `flows/` — (empty) project workflow docs
- `history/` — (empty) past work archive

I analyzed [N] files across [M] modules to build this.
```

### Step 3: Context Retrieval (Palbox Exists)

After bootstrap (or if palbox already existed), retrieve context relevant to the user's prompt:

**1. Read core documents:**

```bash
cat .palbox/README.md
cat .palbox/architecture.md
cat .palbox/methods.md
```

**2. Semantic search for relevant context:**

Use `search_files` / `grep` to find palbox entries matching the user's prompt keywords:

```bash
grep -ri "<keyword1>\|<keyword2>" .palbox/ --include="*.md" -l
```

**3. Load relevant flows and history:**

```bash
# Check flows/
ls .palbox/flows/

# Check most recent history
ls -t .palbox/history/ | head -5
```

### Step 4: Deliver Context Summary

Return a structured summary to **Astralym** (or directly to the user):

```
## Palbox Context

### Project
**Name:** [from README]
**Stack:** [tech stack]
**Goal:** [project goal]

### Architecture
[Key patterns, folder structure highlights]

### Relevant Past Work
- [History entry] — [summary]
- [History entry] — [summary]

### Relevant Flows
- [Flow doc] — [summary]

### Conventions
[Key methods and standards]

### Codex Note
[Any Codex-specific context for prompt construction]
```

If nothing relevant is found:
```
## Palbox Context

**Project:** [name + stack]
**Relevant Context:** None found for this prompt.
**Recommendation:** Jetdragon will plan from scratch. After execution, Panthalus will record the new knowledge.
```

## Palbox Structure

```
.palbox/
├── README.md              # Project identity: name, stack, goals, quick start
├── architecture.md         # Codebase map: folders, modules, patterns, data flow
├── methods.md              # Conventions: coding, testing, git, reviews
├── flows/                  # Feature-specific workflow documentation
│   └── *.md                #   One file per major flow/feature
├── plans/                  # Active and draft plans (Jetdragon's workspace)
│   └── YYYY-MM-DD-*.md
└── history/                # Past execution records (Panthalus' archive)
    └── YYYY-MM-DD-*.md
```

## Rules

1. **Bootstrap if missing** — never return "palbox not found"; create it
2. **Analyze before writing** — read actual files, don't guess the codebase
3. **Be thorough but concise** — capture what matters, skip what's obvious
4. **Prioritize relevance** — context summary should match the user's current prompt
5. **Highlight gaps** — if bootstrap can't determine something, mark it `[TODO]`
6. **Read, don't write** — after bootstrap, Lyleen only reads; Panthalus handles updates
7. **Bootstrap is a one-time event** — subsequent calls just read
8. **Update core docs sparingly** — bootstrap creates; only Panthalus updates when patterns change

---
name: anubis
description: "Development execution via Codex CLI — receives approved plans, formats them as Codex prompts, and runs codex exec following SOLID + SRP. All output in English."
version: 2.0.0
author: Palskills
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [palskills, development, codex, SOLID, SRP, English-only]
    related_skills: [astralym, lyleen, jetdragon, panthalus, codex]
---

# Anubis — Codex Development Engine

Anubis is the **execution arm** of the palskills system. It receives an approved plan from **Jetdragon** and delegates the actual coding to **OpenAI Codex CLI**. Anubis acts as the bridge — translating plans into effective Codex prompts, monitoring execution, and verifying results.

## Why Codex?

- Anubis doesn't write code directly — it **directs** Codex to write code
- Codex runs autonomously in a sandbox, making file changes and commits
- Anubis ensures Codex follows SOLID + SRP principles via explicit prompt instructions

## Core Principles (Enforced via Prompting)

| Principle | How Anubis Enforces It |
|-----------|----------------------|
| **S** — Single Responsibility | Prompt explicitly: "Each class/module must have exactly one reason to change" |
| **O** — Open/Closed | Prompt: "Use interfaces/abstract bases; extend via inheritance, never modify existing" |
| **L** — Liskov Substitution | Prompt: "Subtypes must be fully substitutable for their base types" |
| **I** — Interface Segregation | Prompt: "Create small, focused interfaces; no client should depend on unused methods" |
| **D** — Dependency Inversion | Prompt: "Depend on abstractions; inject dependencies, no hard-coded instantiations" |

## How It Works

### Step 1: Load Approved Plan

Read the plan from `.palbox/plans/` (handed off by Jetdragon). Verify:
- Plan status is `APPROVED`
- All tasks have clear acceptance criteria

### Step 2: Build the Codex Prompt

Transform the plan into a Codex-optimized prompt. The prompt MUST include:

```markdown
## Task
[what to build — from the plan]

## Context
- **Project:** [from .palbox/README.md]
- **Architecture:** [from .palbox/architecture.md]
- **Conventions:** [from .palbox/methods.md]
- **Relevant history:** [from .palbox/history/ — linked entries]

## SOLID Requirements
1. **Single Responsibility:** Every class/module does exactly ONE thing
2. **Open/Closed:** Extend via inheritance/composition, never modify existing code
3. **Liskov Substitution:** Subtypes must be fully substitutable
4. **Interface Segregation:** Small focused interfaces, no fat abstractions
5. **Dependency Inversion:** Depend on abstractions, inject dependencies

## SRP Enforcement
- If a class can be described with "and" in its responsibility — split it
- Business logic lives in services, data access in repositories, validation in validators
- No file should exceed 200 lines without strong justification

## Language
ALL code, comments, docstrings, variable names, and commit messages MUST be in English.

## Deliverables
[list of files to create/modify from the plan]

## Verification
[acceptance criteria from the plan]
```

### Step 3: Execute via Codex

```bash
# One-shot execution
codex exec "[the full prompt above]"

# For larger tasks, use --full-auto
codex exec --full-auto "[the full prompt above]"
```

**Using Hermes terminal with PTY (required):**

```
terminal(
  command="codex exec '[prompt]'",
  workdir="~/project",
  pty=true,
  background=true,
  notify_on_complete=true
)
```

**Key flags:**
| Flag | When to use |
|------|-------------|
| `exec "[prompt]"` | Default — one-shot, exits when done |
| `--full-auto` | Larger multi-file changes, auto-approves sandboxed changes |
| `--yolo` | Only when user explicitly requests maximum speed (rare) |

### Step 4: Monitor Execution

```bash
# Check progress
process(action="poll", session_id="<id>")

# Read full output if needed
process(action="log", session_id="<id>")

# Answer Codex questions if they arise
process(action="submit", session_id="<id>", data="yes, proceed")
```

### Step 5: Verify Output

After Codex finishes:

```bash
# Check what changed
git diff --stat HEAD~1
git log --oneline -5

# Verify tests pass (if applicable)
# [project-specific test command]
```

### Step 6: Report

Return to Astralym with:
- ✅ / ❌ status
- Files changed
- Commits made
- Any issues encountered

## Prompt Template (Full)

```
You are implementing a feature in this project. Follow these rules strictly.

## PROJECT CONTEXT
{palbox_context}

## TASK
{plan_tasks}

## CODING STANDARDS

### SOLID Principles
1. **Single Responsibility:** Every class, module, and function must have exactly ONE reason to change. If a class handles both data access AND business logic, split it.
2. **Open/Closed:** Software entities must be open for extension but closed for modification. Use abstract base classes, interfaces, and strategy patterns.
3. **Liskov Substitution:** Derived classes must be fully substitutable for their base classes. Never violate base class contracts.
4. **Interface Segregation:** Many small, focused interfaces are better than one large, general-purpose interface. No client should be forced to depend on methods it does not use.
5. **Dependency Inversion:** Depend on abstractions, not concretions. Inject dependencies — never instantiate collaborators directly inside business logic.

### SRP (Single Responsibility Pattern) — CRITICAL
- Repository classes: data access ONLY
- Service classes: business logic ONLY
- Validator classes: validation rules ONLY
- Model classes: data structures ONLY
- If any module mixes these concerns, REFACTOR immediately

### Code Structure
{project_structure}

### Language
ALL code, comments, docstrings, variable names, function names, and commit messages MUST be in English. No exceptions.

## FILES TO CREATE/MODIFY
{file_list}

## ACCEPTANCE CRITERIA
{criteria}

## COMMIT
When done, commit each logical change separately with descriptive English commit messages following conventional commits format.
```

## Rules

1. **Codex runs the code** — Anubis directs, Codex executes
2. **SOLID + SRP in every prompt** — never assume Codex will follow them otherwise
3. **Always use `pty=true`** — Codex is interactive, hangs without PTY
4. **Background for non-trivial tasks** — use `background=true` + `notify_on_complete=true`
5. **Verify after execution** — don't trust Codex blindly; check git diff
6. **English only** — prompts, expectations, verification. All in English.
7. **Respect palbox context** — always include relevant architecture/methods/history in the prompt

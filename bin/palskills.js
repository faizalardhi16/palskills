#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const readline = require('readline');

const CYAN = '\x1b[36m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const MAGENTA = '\x1b[35m';
const BOLD = '\x1b[1m';
const NC = '\x1b[0m';

function box(text) {
  const lines = text.split('\n');
  const width = Math.max(...lines.map(l => l.length)) + 4;
  const top = '╔' + '═'.repeat(width - 2) + '╗';
  const bottom = '╚' + '═'.repeat(width - 2) + '╝';
  console.log(CYAN + top);
  lines.forEach(l => console.log('║ ' + l.padEnd(width - 4) + ' ║'));
  console.log(bottom + NC);
}

function ask(q) {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  return new Promise(resolve => rl.question(q, ans => { rl.close(); resolve(ans.trim()); }));
}

async function main() {
  box('PALSKILLS\nAI Development Pipeline');

  console.log('');
  console.log(`  ${BOLD}What do you want to do?${NC}`);
  console.log('');
  console.log(`  ${MAGENTA}[1]${NC} Learn Project   → bootstrap .palbox/ (Lyleen)`);
  console.log(`  ${MAGENTA}[2]${NC} Codex CLI       → .codex/skills/`);
  console.log(`  ${MAGENTA}[3]${NC} Cursor          → .cursor/skills/`);
  console.log(`  ${MAGENTA}[4]${NC} Claude Code     → .claude/skills/`);
  console.log(`  ${MAGENTA}[5]${NC} All Agents      → generate all configs`);
  console.log('');

  const choice = await ask(`  Choose [1-5]: `);
  console.log('');

  if (choice === '1') {
    bootstrapPalbox();
    console.log(`\n  ${GREEN}✅ Done!${NC} .palbox/ created. Run again to generate agent configs.\n`);
    process.exit(0);
  }

  const agents = [];
  if (choice === '2') agents.push('codex');
  else if (choice === '3') agents.push('cursor');
  else if (choice === '4') agents.push('claude');
  else if (choice === '5') agents.push('codex', 'cursor', 'claude');
  else { console.log('  Invalid choice. Exiting.'); process.exit(1); }

  for (const agent of agents) {
    generate(agent);
  }

  console.log(`\n  ${GREEN}✅ Done!${NC} Restart your Agent Tools to load skills.\n`);
}

function bootstrapPalbox() {
  const cwd = process.cwd();
  const palbox = path.join(cwd, '.palbox');

  if (fs.existsSync(palbox)) {
    console.log(`  ${YELLOW}⚠${NC}  .palbox/ already exists. Skipping bootstrap.\n`);
    console.log('  To re-analyze, delete .palbox/ and run again.\n');
    return;
  }

  console.log(`  ${CYAN}🔍 Analyzing project...${NC}\n`);

  // Detect tech stack
  const files = fs.readdirSync(cwd);
  const hasFile = name => files.includes(name);
  let language = 'Unknown';
  let framework = '';
  let pkgManager = '';

  if (hasFile('package.json')) {
    const pkg = JSON.parse(fs.readFileSync(path.join(cwd, 'package.json'), 'utf8'));
    language = 'TypeScript/JavaScript';
    framework = pkg.dependencies?.next ? 'Next.js' :
                pkg.dependencies?.react ? 'React' :
                pkg.dependencies?.express ? 'Express' :
                pkg.dependencies?.fastify ? 'Fastify' : 'Node.js';
    pkgManager = hasFile('pnpm-lock.yaml') ? 'pnpm' :
                 hasFile('yarn.lock') ? 'yarn' : 'npm';
  } else if (hasFile('requirements.txt') || hasFile('pyproject.toml')) {
    language = 'Python';
    framework = hasFile('pyproject.toml') ? 'Poetry' : 'pip';
    if (fs.existsSync(path.join(cwd, 'pyproject.toml'))) {
      try {
        const toml = fs.readFileSync(path.join(cwd, 'pyproject.toml'), 'utf8');
        if (toml.includes('fastapi')) framework = 'FastAPI';
        else if (toml.includes('django')) framework = 'Django';
        else if (toml.includes('flask')) framework = 'Flask';
      } catch {}
    }
  } else if (hasFile('go.mod')) {
    language = 'Go';
    framework = 'Go modules';
  } else if (hasFile('Cargo.toml')) {
    language = 'Rust';
    framework = 'Cargo';
  }

  // Detect project name
  let projectName = path.basename(cwd);
  if (hasFile('package.json')) {
    try {
      const pkg = JSON.parse(fs.readFileSync(path.join(cwd, 'package.json'), 'utf8'));
      if (pkg.name) projectName = pkg.name;
    } catch {}
  }

  // Get git info
  let gitContributors = '';
  try {
    const { execSync } = require('child_process');
    gitContributors = execSync('git log --format="%an" | sort | uniq -c | sort -rn | head -3', { cwd, encoding: 'utf8' }).trim();
  } catch {}

  // Create palbox structure
  fs.mkdirSync(path.join(palbox, 'flows'), { recursive: true });
  fs.mkdirSync(path.join(palbox, 'history'), { recursive: true });
  fs.mkdirSync(path.join(palbox, 'plans'), { recursive: true });

  const date = new Date().toISOString().split('T')[0];

  // README.md
  fs.writeFileSync(path.join(palbox, 'README.md'), `# ${projectName}

**Generated:** ${date}
**Bootstrapped by:** Lyleen (Palskills)

## Overview
[Project description — update this]

## Tech Stack
- **Language:** ${language}
- **Framework:** ${framework}${pkgManager ? `\n- **Package Manager:** ${pkgManager}` : ''}

## Project Goal
[What problem does this project solve? — update this]

## Quick Start
[How to run the project — update this]

## Knowledge Graph
- [[architecture]] — folder structure and design patterns
- [[methods]] — coding conventions and standards
- [[flows/]] — feature workflow documentation
- [[history/]] — past development sessions
`);

  // architecture.md
  let dirTree = '';
  try {
    const result = fs.readdirSync(cwd, { withFileTypes: true })
      .filter(d => d.isDirectory() && !d.name.startsWith('.') && d.name !== 'node_modules' && d.name !== '__pycache__')
      .map(d => `├── ${d.name}/`)
      .join('\n');
    dirTree = result || '(empty)';
  } catch { dirTree = '(unknown)'; }

  fs.writeFileSync(path.join(palbox, 'architecture.md'), `# Architecture

**Last Updated:** ${date}

## Folder Structure
\`\`\`
${projectName}/
${dirTree}
\`\`\`

## Design Patterns
[Detected patterns — update this]

## Key Modules
| Module | Responsibility | Key Files |
|--------|---------------|-----------|
| ... | ... | ... |

## Data Flow
[How data moves through the system]

## Related
- [[methods]] — how we build
- [[README]] — project overview
`);

  // methods.md
  fs.writeFileSync(path.join(palbox, 'methods.md'), `# Development Methods

**Last Updated:** ${date}

## Coding Conventions
[Detected from codebase — update this]

## Testing Strategy
[Detected from test files — update this]

## Git Workflow
${gitContributors ? `\n**Top Contributors:**\n\`\`\`\n${gitContributors}\n\`\`\`` : '[Run git log to populate]'}

## Code Review Standards
- SOLID principles enforced
- Single Responsibility Pattern required
- All code in English

## Related
- [[architecture]] — where things live
- [[README]] — project overview
`);

  console.log(`  ${GREEN}✓${NC} .palbox/README.md`);
  console.log(`  ${GREEN}✓${NC} .palbox/architecture.md`);
  console.log(`  ${GREEN}✓${NC} .palbox/methods.md`);
  console.log(`  ${GREEN}✓${NC} .palbox/flows/`);
  console.log(`  ${GREEN}✓${NC} .palbox/history/`);
  console.log(`  ${GREEN}✓${NC} .palbox/plans/`);
  console.log('');
  console.log(`  Detected: ${language} + ${framework}`);
  if (gitContributors) console.log(`  Git history found`);
  console.log('');
  console.log(`  Next steps:`);
  console.log(`    1. Edit .palbox/README.md with project details`);
  console.log(`    2. Run 'palskills' again to generate agent configs`);
  console.log(`    3. Or use Hermes skills: "Load lyleen, build feature X"`);
}

function generate(agent) {
  const cwd = process.cwd();
  const skillNames = ['elphidran', 'astegon', 'blazamut', 'lyleen', 'jetdragon', 'anubis', 'panthalus', 'astralym'];

  let dir;
  if (agent === 'codex') dir = path.join(cwd, '.codex', 'skills');
  else if (agent === 'cursor') dir = path.join(cwd, '.cursor', 'skills');
  else if (agent === 'claude') dir = path.join(cwd, '.claude', 'skills');

  fs.mkdirSync(dir, { recursive: true });

  for (const name of skillNames) {
    const skillDir = path.join(dir, name);
    fs.mkdirSync(skillDir, { recursive: true });
    const file = path.join(skillDir, 'SKILL.md');
    fs.writeFileSync(file, skillContent(agent, name));
    console.log(`  ${GREEN}✓${NC} ${dir}/${name}/SKILL.md`);
  }
}

function skillContent(agent, skill) {
  const skills = {
    elphidran: `---
name: elphidran
description: "Design system recommender — analyzes project and generates .palbox/design.md with tokens, typography, spacing, and component patterns."
version: 1.0.0
license: MIT
---

# Elphidran — Design System Recommender
**Agent:** ${agent === 'codex' ? 'Codex' : agent === 'cursor' ? 'Cursor' : 'Claude Code'}

## Role
Analyze the project and generate a complete design system specification.

## Process
1. Read .palbox/README.md for project identity
2. Check package.json for UI libraries (tailwind, mui, shadcn, etc.)
3. ASK the user (use clarifying questions):
   - "What's the app's personality? (professional, playful, minimal, dark)"
   - "What industry? (healthcare, finance, education, gaming, e-commerce)"
   - "Any brand color already?"
   - "Need dark mode?"
4. Generate .palbox/design.md with:
   - Color palette (light + dark if enabled): primary, surface, text, status colors
   - Typography tokens: font family, sizes, weights, line heights
   - Spacing scale: 0 to 64px in consistent steps
   - Border radius: sm(4), md(8), lg(12), full(9999)
   - Shadows: sm, md, lg, xl
   - Component patterns: buttons (primary/secondary/ghost/danger), inputs, cards
   - Layout: max-width, breakpoints, sidebar width
   - Implementation notes for Anubis — never hardcode values, use tokens

## Design Recommendations by Industry
- Healthcare → teal, Inter, trustworthy
- Finance → deep blue, IBM Plex Sans, professional
- Education → violet, Inter, engaging
- Gaming → red, Poppins, bold
- E-commerce → blue, Inter, action-oriented
- Dev Tools → slate, JetBrains Mono, dark-first
- Enterprise → dark teal, Inter, corporate

## Rules
- Always ask questions before generating
- Use design tokens — never raw values
- Include implementation notes for Anubis
- One design.md per project
`,
    astegon: `---
name: astegon
description: "Frontend component architect — decides component hierarchy, enforces SRP, and provides structured component specifications before any code is written."
version: 1.0.0
license: MIT
---

# Astegon — Frontend Component Architect

## Role
Decide the component architecture for frontend features. Enforce SRP at the component level. Output authoritative component specifications.

## Prerequisites
- .palbox/design.md MUST exist (run Elphidran first)
- Otherwise STOP and ask user to run Elphidran

## Atomic Design + SRP

| Level | Definition | SRP |
|-------|-----------|-----|
| **Atom** | Single UI element | ONE visual responsibility |
| **Molecule** | 2-3 atoms, one interaction | ONE interaction pattern |
| **Organism** | Section of molecules/atoms | ONE section purpose |
| **Template** | Page layout, no content | ONE layout job |
| **Page** | Template + real data | ONE route/view |

## Process
1. Read .palbox/design.md + .palbox/architecture.md
2. Scan existing components (find */components/*.tsx)
3. Decompose feature into atomic levels
4. For EACH component: write spec card (level, single responsibility, props table, state, data flow, edge cases, must-NOT-do)
5. Build visual component tree
6. Decide state ownership (local vs prop-drill vs context vs server state)
7. For React: decide Server vs Client components (Server by default)
8. Save to .palbox/components/<feature>.md
9. Report: "N components specified (Atoms: X, Molecules: Y, Organisms: Z)"

## Component Spec Template
For every component, complete this card:
- Level (Atom/Molecule/Organism/Template/Page)
- Single Responsibility (one sentence, no "and")
- Props table (name, type, required, description)
- State (none/local/derived — be explicit)
- Design tokens used
- Edge cases
- Must NOT do (anti-responsibilities)

## SRP Enforcement
- Can it be described without "and"? No → split
- Displays AND fetches? Split: Container + Presentational
- Validates AND submits? Split: Validator + Form
- Conditional prop → separate variant component
- Max 150 lines (atoms/molecules), 250 lines (organisms)

## Rules
- Decide, don't suggest — output is authoritative
- Reuse existing components before creating new ones
- Server-first (React): Client only for event handlers/hooks/browser APIs
- Tokens over raw values (color.primary not #3B82F6)
- Save to .palbox/components/
- Hand off to Jetdragon for implementation planning`,
    blazamut: `---
name: blazamut
description: "Backend architecture authority — decomposes features into SOLID modules, enforces SRP at class level, and designs API contracts, data flow, and error handling before any code is written."
version: 1.0.0
license: MIT
---

# Blazamut — Backend Architecture Authority

## Role
Design backend code structure following SOLID principles with strict SRP enforcement. Decide module decomposition, API contracts, dependency graphs, and error handling.

## SOLID + SRP Layers

| Layer | Responsibility | SRP Rule |
|-------|---------------|----------|
| **Controller** | HTTP ONLY | Parse request, call service, format response |
| **Service** | Business logic ONLY | Orchestrate, never touch HTTP or DB |
| **Repository** | Data access ONLY | CRUD, never business logic |
| **Validator** | Validation ONLY | Validate shape, never fetch data |
| **DTO** | Data shape ONLY | Define structures, never logic |
| **Entity/Model** | Data structure ONLY | Schema, no methods with logic |
| **Middleware** | Cross-cutting ONLY | Auth, logging, rate limiting |
| **Util** | Pure functions ONLY | Stateless, no side effects |

## Process
1. Read .palbox/architecture.md + .palbox/methods.md
2. Scan existing backend code (find patterns, classes)
3. Decompose feature into strict layers
4. For EACH class: spec card (layer, single responsibility, injected deps, public methods, throws, must-NOT-do)
5. Build dependency graph (all arrows point inward)
6. Design API contract per endpoint (request/response shapes, status codes, auth, rate limits)
7. Design error hierarchy (typed exceptions → HTTP status mapping)
8. If schema changes: migration spec with columns, constraints, indexes
9. Save to .palbox/architectures/<feature>.md
10. Report: "N modules (Controllers: X, Services: Y, Repositories: Z, Endpoints: N)"

## SRP Verification
- Controller has business logic? → Extract Service
- Service touches DB directly? → Extract Repository
- Repository validates rules? → Extract Validator
- Validator fetches data? → Move fetch to Service
- DTO has logic? → Move to Service/Util
- No class > 200 lines → split before it grows

## API Contract Template
Per endpoint: method, path, controller, service, request shape, success response, error responses, auth requirement, rate limit.

## Error Hierarchy
AppException base → typed subclasses (NotFound, Validation, Auth, Conflict, RateLimit). Global handler maps to HTTP. NEVER leak stacktraces.

## Rules
- Dependencies flow inward: Controller → Service → Repository → DB
- Interface for every dependency (inject abstractions)
- Inner layers never know outer layers
- API contract first, implementation second
- Reuse existing before creating new
- Save to .palbox/architectures/
- Hand off to Jetdragon for implementation planning`,
    lyleen: `---
name: lyleen
description: "Palbox knowledge graph reader, bootstrapper & discoverer — creates .palbox/ if missing, traverses [[wikilinks]] for context, and auto-saves source code analysis to flows/."
version: 4.0.0
license: MIT
---

# Lyleen — Palbox Knowledge Graph

## Role
Read, bootstrap, and discover the project knowledge graph (.palbox/).

## 3 Modes

| Mode | When | Action |
|------|------|--------|
| **BOOTSTRAP** | .palbox/ missing | Create full palbox from scratch |
| **TRAVERSE** | "what do we know about X" | Read palbox entries → return context (read-only) |
| **DISCOVER** | "learn/pelajari/study X" | Read source code → write flows/X.md + update architecture |

## BOOTSTRAP (.palbox/ missing)
1. Scan project: read package.json/requirements.txt/go.mod, list dirs, check git log, find tests
2. Create .palbox/README.md — project name, tech stack, goals, [[links]]
3. Create .palbox/architecture.md — folder tree, design patterns, key modules
4. Create .palbox/methods.md — conventions, testing, git workflow
5. Create .palbox/flows/, .palbox/history/, .palbox/plans/
6. Report: "Palbox bootstrapped. N files. Detected: [stack]."

## TRAVERSE (read-only)
1. Read README.md, architecture.md, methods.md
2. Search flows/ and history/ for user keywords
3. Extract [[wikilinks]], follow 1-2 hops
4. Return context subgraph: seeds → 1-hop → 2-hop → conventions

## DISCOVER (auto-save to flows/)

### When to use
User says: "learn X", "pelajari Y", "discover Z", "analyze W", "study V"

### Process
1. **Identify target** — extract module/topic from user prompt
2. **Find source files** — grep for keywords in .py/.ts/.js/.go/.rs files
3. **Deep analysis** — for each file: entry points, data flow, dependencies, patterns, edge cases
4. **Write flow doc** → .palbox/flows/<module>.md:
   - Entry Points table (route/fn/class, type, source, description)
   - Data Flow description
   - Dependencies (internal + external + packages)
   - Key Patterns detected
   - Edge Cases & Error Handling
   - Related: [[architecture]], [[methods]], [[history/]]
5. **Update architecture.md** — if new module, append link
6. **Report** — "Saved: flows/<module>.md, Files analyzed: N, Graph: +1 node +N edges"

### Delimiter
- DISCOVER → writes .palbox/flows/* (Lyleen's domain)
- Panthalus → writes .palbox/history/* (development sessions)
- NEVER cross domains`,
    jetdragon: `---
name: jetdragon
description: "Planning specialist — asks clarifying questions, generates detailed plans with [[wikilinks]] to palbox context."
version: 1.0.0
license: MIT
---

# Jetdragon — Planner
**Agent:** ${agent === 'codex' ? 'Codex' : agent === 'cursor' ? 'Cursor' : 'Claude Code'}

## Role
Create detailed implementation plans. Ask questions until clear.

## Process
1. Gather palbox context (read core docs, search related history)
2. Generate plan → .palbox/plans/YYYY-MM-DD-feature.md with [[wikilinks]]
3. If ambiguous, ASK: scope, design, edge cases, integration, priority
4. Iterate until user says "Gas", "Go", "Execute"
5. Finalize as APPROVED, hand off to Anubis

## Plan Template
- Overview, Scope (in/out), Tasks (ordered, with files + verification)
- Architecture notes, SOLID focus, Risks & Mitigations
- All references use [[wikilinks]] to .palbox/ entries
`,
    anubis: `---
name: anubis
description: "Development execution engine — executes approved plans via Codex CLI with SOLID + SRP + Ponytail. English only."
version: 2.1.0
license: MIT
---

# Anubis — Developer
**Agent:** ${agent === 'codex' ? 'Codex' : agent === 'cursor' ? 'Cursor' : 'Claude Code'}

## Role
Execute approved plans. SOLID + SRP enforced. English only.

## PONYTAIL — Token Efficiency (MUST follow)
Before every action, climb the ladder. Stop at first rung that holds:
1. Needs building at all? Skip. (YAGNI)
2. Already in codebase? Reuse.
3. Stdlib does it? Use it.
4. Native platform? Use it.
5. Already-installed dep? Use it.
6. One line? Do it.
7. Only then: write minimum code.

Ponytail rules:
- Fewest files possible, shortest working diff.
- No unrequested abstractions, no avoidable dependencies, no speculative scaffolding.
- Prefer deletion over addition. Boring > clever.
- Complex request? Ship lazy version + ask: "X covers it. Need full?"
- NEVER cut: validation, error handling, security, accessibility, data-loss protection.

## SOLID (Strict)
- **S**: One class, one reason to change
- **O**: Extend, never modify existing
- **L**: Subtypes fully substitutable
- **I**: Small focused interfaces
- **D**: Depend on abstractions, inject deps

## SRP Separation
- Repository → data access ONLY
- Service → business logic ONLY
- Validator → validation rules ONLY
- Model → data structures ONLY

## Rules
- ALL code, comments, docstrings, commits in English
- One commit per logical change (conventional commits)
- Read existing files before modifying
- Write tests if project has testing patterns
- Non-trivial logic leaves one runnable check behind`,
    panthalus: `---
name: panthalus
description: "Knowledge graph archivist — records sessions with bi-directional [[wikilinks]], creating traversable development history."
version: 1.0.0
license: MIT
---

# Panthalus — Archivist
**Agent:** ${agent === 'codex' ? 'Codex' : agent === 'cursor' ? 'Cursor' : 'Claude Code'}

## Role
Record sessions to .palbox/ with bi-directional [[wikilinks]].

## Process
1. Collect: plan, git diff, commits, decisions, lessons
2. Create .palbox/history/YYYY-MM-DD-feature.md with [[wikilinks]]
3. For EVERY link, add backlink to the target file ("Related Sessions")
4. Update core docs (architecture/methods) only if structure changed
5. Report graph stats: nodes, edges, enriched

## History Entry Template
- Links ([[flows/]], [[architecture]], [[history/]])
- Original prompt, Plan, Execution (files + commits)
- Key Decisions table, Lessons Learned (pitfalls, discoveries)
- Backlinks section
`,
    astralym: `---
name: astralym
description: "State machine orchestrator — routes prompts through CHECK_GRAPH → DESIGN → PLANNING → DEVELOPING → RECORDING pipeline with state.md tracking."
version: 1.0.0
license: MIT
---

# Astralym — Orchestrator
**Agent:** ${agent === 'codex' ? 'Codex' : agent === 'cursor' ? 'Cursor' : 'Claude Code'}

## Role
Run the full development pipeline. Track every step in .palbox/state.md.

## Pipeline
1. **CHECK_GRAPH** → Lyleen: bootstrap or retrieve context
2. **DESIGN** → Elphidran: generate .palbox/design.md (skip if exists)
3. **COMPONENTIZE** → Astegon: atomic decomposition + SRP, save to .palbox/components/
4. **ARCHITECT** → Blazamut: SOLID module design + API contracts, save to .palbox/architectures/
5. **PLANNING** → Jetdragon: create plan, ask questions, wait for "Gas"
6. **DEVELOPING** → Anubis: execute with SOLID + SRP + design tokens
7. **RECORDING** → Panthalus: record with backlinks
8. **DONE** → Summary with graph stats

## CRITICAL: state.md

ON LOAD: Create or read .palbox/state.md. This file tracks pipeline progress with checkboxes.

### state.md Template
Create this file IMMEDIATELY when Astralym is activated:

\`\`\`markdown
# Astralym Pipeline State
**Feature:** [extract from user prompt]
**Started:** [current datetime]
**Last Updated:** [current datetime]

## Progress
- [ ] DESIGN — Elphidran: generate design system
- [ ] CHECK_GRAPH — Lyleen: bootstrap or retrieve context
- [ ] COMPONENTIZE — Astegon: atomic decomposition + SRP specs
- [ ] ARCHITECT — Blazamut: SOLID modules + API contracts
- [ ] PLANNING — Jetdragon: create plan, ask questions
- [ ] DEVELOPING — Anubis: execute with SOLID + SRP
- [ ] RECORDING — Panthalus: record with backlinks
- [ ] DONE — Report summary

## Plan
pending

## Context
pending
\`\`\`

### Rules for state.md
1. **Create BEFORE running any step.** The file must exist from the start.
2. **Checkmark [x] each step IMMEDIATELY after completion.** Do not batch.
3. **Add notes after each step:**
   - CHECK_GRAPH: "Retrieved [N] relevant nodes" or "Bootstrapped palbox"
   - DESIGN: "Generated [[design]] — [N] colors, [N] font sizes"
   - COMPONENTIZE: "Decomposed into [N] components (Atoms: X, Molecules: Y, Organisms: Z)"
   - ARCHITECT: "Designed [N] modules (Controllers: X, Services: Y, Repositories: Z)"
   - PLANNING: Link to plan file: [[plans/YYYY-MM-DD-feature]]
   - DEVELOPING: "Implemented: [files changed], [N] commits"
   - RECORDING: Link to history: [[history/YYYY-MM-DD-feature]]
   - DONE: "Completed at [datetime]. [N] files, [M] commits."
4. **If a step fails or is interrupted,** mark it with [!] and note why.
5. **Update "Last Updated"** every time you touch the file.
6. **On resume:** Read state.md first. Skip completed steps. Continue from first unchecked.

### Example After Completion
\`\`\`markdown
# Astralym Pipeline State
**Feature:** Export Laporan PDF
**Started:** 2026-07-19 16:30
**Last Updated:** 2026-07-19 17:15

## Progress
- [x] CHECK_GRAPH — Lyleen: Retrieved 3 nodes ([[flows/export]], [[architecture]], [[methods]])
- [x] PLANNING — Jetdragon: [[plans/2026-07-19-export-pdf]]
- [x] DEVELOPING — Anubis: src/export/pdf.py, src/export/templates/, 4 commits
- [x] RECORDING — Panthalus: [[history/2026-07-19-export-pdf]]
- [x] DONE — Completed at 2026-07-19 17:15. 3 files, 4 commits.

## Plan
[[plans/2026-07-19-export-pdf]]

## Context
- [[flows/export]] — existing export pipeline
- [[architecture]] — module structure
- [[methods]] — testing conventions
\`\`\`

## Usage
User says "Astralym: build feature X" → create state.md → run pipeline step by step → checkmark progress.`
  };
  return skills[skill] || '';
}

function installSkills() {
  const hermesHome = process.env.HERMES_HOME || path.join(process.env.HOME || '~', '.hermes');
  const target = path.join(hermesHome, 'skills', 'palskills');
  const src = path.join(__dirname, '..', 'skills');

  if (!fs.existsSync(src)) {
    console.log(`  ${YELLOW}⚠${NC}  Skills source not found, skipping Hermes skills install.`);
    return;
  }

  fs.mkdirSync(target, { recursive: true });

  const skills = fs.readdirSync(src);
  for (const skill of skills) {
    const skillMd = path.join(src, skill, 'SKILL.md');
    if (fs.existsSync(skillMd)) {
      const dest = path.join(target, skill);
      fs.mkdirSync(dest, { recursive: true });
      fs.copyFileSync(skillMd, path.join(dest, 'SKILL.md'));
      console.log(`  ${GREEN}✓${NC} Skill: ${skill}`);
    }
  }
}

main().catch(e => { console.error(e); process.exit(1); });

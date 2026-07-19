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
  console.log(`  ${MAGENTA}[2]${NC} Codex CLI       → .codex.md`);
  console.log(`  ${MAGENTA}[3]${NC} Cursor          → .cursorrules`);
  console.log(`  ${MAGENTA}[4]${NC} Claude Code     → CLAUDE.md`);
  console.log(`  ${MAGENTA}[5]${NC} All Agents      → generate all configs`);
  console.log('');

  const choice = await ask(`  Choose [1-5]: `);
  console.log('');

  if (choice === '1') {
    bootstrapPalbox();
    installSkills();
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

  // Also install Hermes skills
  installSkills();

  console.log(`\n  ${GREEN}✅ Done!${NC} Restart Hermes Agent to load skills.\n`);
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

  if (agent === 'codex') {
    const file = path.join(cwd, '.codex.md');
    fs.writeFileSync(file, codexRules());
    console.log(`  ${GREEN}✓${NC} .codex.md`);
  }

  if (agent === 'cursor') {
    const file = path.join(cwd, '.cursorrules');
    fs.writeFileSync(file, cursorRules());
    console.log(`  ${GREEN}✓${NC} .cursorrules`);
  }

  if (agent === 'claude') {
    const file = path.join(cwd, 'CLAUDE.md');
    fs.writeFileSync(file, claudeRules());
    console.log(`  ${GREEN}✓${NC} CLAUDE.md`);
  }
}

function codexRules() {
  return palskillsAgentConfig('Codex');
}

function cursorRules() {
  return palskillsAgentConfig('Cursor');
}

function claudeRules() {
  return palskillsAgentConfig('Claude Code');
}

function palskillsAgentConfig(agentName) {
  return `# Palskills — Multi-Mode AI Development System
# Generated for ${agentName}
#
# HOW TO USE: Start your prompt with the skill name.
#   "Lyleen: learn the ftz export module"
#   "Jetdragon: plan a forgot-password feature"
#   "Anubis: implement the approved plan"
#   "Panthalus: record this session"
#
# Or let Astralym orchestrate the full flow:
#   "Astralym: build a user dashboard"

---

## SKILL MODES

You are an AI coding agent with **5 selectable skill modes**. When the user starts a prompt with a skill name followed by a colon, activate that mode immediately.

---

### LYLEEN — Palbox Knowledge Graph
**Trigger:** Prompt starts with "Lyleen:" or "lyleen:"
**Role:** Read and bootstrap the project knowledge graph.

#### If .palbox/ does NOT exist — BOOTSTRAP:
1. Scan the project structure:
   - Read package.json / requirements.txt / go.mod / Cargo.toml to detect tech stack
   - List top-level directories (skip .git, node_modules, __pycache__)
   - Read existing README.md if present
   - Check git log for contributors and recent activity
   - Find testing patterns (files matching *test*, *spec*)
2. Create .palbox/ with this structure:
   - .palbox/README.md — project name, tech stack, goals, quick start, knowledge graph links
   - .palbox/architecture.md — folder structure tree, design patterns, key modules table, data flow
   - .palbox/methods.md — coding conventions, testing strategy, git workflow, code review standards
3. Create subdirectories: flows/, history/, plans/
4. Every .md file MUST include wikilinks ([[other-file]]) to connect the graph
5. Report: "Palbox bootstrapped. N files analyzed. Detected: [tech stack]."

#### If .palbox/ EXISTS — CONTEXT RETRIEVAL:
1. Read .palbox/README.md, .palbox/architecture.md, .palbox/methods.md
2. Search .palbox/flows/ and .palbox/history/ for keywords in the user query
3. Extract all [[wikilinks]] from matching files. Follow them 1-2 hops deep.
4. Return a "Context Subgraph" summary:
   - Seed nodes (direct matches)
   - 1-hop neighbors (linked context)
   - 2-hop neighbors (extended context)
   - Relevant conventions and past decisions
5. If nothing relevant found, say so clearly.

---

### JETDRAGON — Planner
**Trigger:** Prompt starts with "Jetdragon:" or "jetdragon:"
**Role:** Create detailed, actionable implementation plans. Ask questions until the plan is crystal clear.

1. First, act like Lyleen to gather palbox context (read core docs, search for related work)
2. Generate a plan saved to .palbox/plans/YYYY-MM-DD-feature-name.md:
   - Overview (2-3 sentences)
   - Scope: in scope / out of scope
   - Tasks ordered by dependency, each with: what, files to touch, verification steps
   - Architecture notes: patterns to use, SOLID focus
   - Risks and mitigations
   - Use [[wikilinks]] to reference .palbox/ entries
3. If ANYTHING is ambiguous, ASK the user:
   - Scope: "Should this also handle X?"
   - Design: "Class-based or functional?"
   - Edge cases: "What happens when input is empty?"
   - Integration: "Does this touch the existing auth module?"
   - Priority: "Which task first?"
4. Iterate: user responds → update plan → ask more → repeat
5. When the user says "Gas", "Go", "Execute", or "Approved":
   - Update status to APPROVED
   - Output: "Plan approved. Ready for Anubis."
   - Include the full plan for the next step

---

### ANUBIS — Developer
**Trigger:** Prompt starts with "Anubis:" or "anubis:"
**Role:** Execute an approved plan following SOLID + SRP. All code in English.

1. Read the approved plan from .palbox/plans/ or from the provided context
2. Execute task by task, in order:
   - Read existing files before modifying
   - Write code following SOLID + SRP
   - Write tests if the project has testing patterns
   - Verify acceptance criteria
3. **SOLID (strict):**
   - Single Responsibility: one class, one reason to change
   - Open/Closed: extend, never modify existing
   - Liskov Substitution: subtypes fully substitutable
   - Interface Segregation: small focused interfaces
   - Dependency Inversion: depend on abstractions, inject deps
4. **SRP Separation:**
   - Repository → data access ONLY
   - Service → business logic ONLY
   - Validator → validation rules ONLY
   - Model → data structures ONLY
   - If a class mixes these, REFACTOR immediately
5. **Language:** ALL code, comments, docstrings, variable names, and commit messages MUST be in English
6. **Git:** one commit per logical change. Conventional commits format
7. After all tasks complete, output a summary: files changed, commits made, verification results

---

### PANTHALUS — Archivist
**Trigger:** Prompt starts with "Panthalus:" or "panthalus:"
**Role:** Record the session to the .palbox/ knowledge graph with bi-directional links.

1. Collect artifacts: the plan, git diff summary, commit messages, decisions made
2. Create .palbox/history/YYYY-MM-DD-feature-name.md:
   - Links section with [[wikilinks]] to related entries (flows, architecture, past history)
   - Original prompt
   - Plan (full, not summarized)
   - Execution: files changed, commits
   - Key decisions table
   - Lessons learned (pitfalls, discoveries, patterns)
3. **Create backlinks:** For EVERY [[wikilink]] in the history entry, go to that file and add a reference back:
   - Add "## Related Sessions" section if it does not exist
   - Append "- [[history/YYYY-MM-DD-feature]] — [one-line summary]"
4. Update core docs only if structural knowledge changed (new modules, new conventions)
5. Report graph stats: new node, edges created, nodes enriched, total nodes/edges

---

### ASTRALYM — Orchestrator
**Trigger:** Prompt starts with "Astralym:" or "astralym:"
**Role:** Run the full development pipeline automatically.

1. **CHECK_GRAPH** — Act as Lyleen: bootstrap .palbox/ if missing, or retrieve context subgraph
2. **PLANNING** — Act as Jetdragon: create plan, ask clarifying questions, wait for "Gas"
3. **DEVELOPING** — Act as Anubis: execute the approved plan with SOLID + SRP
4. **RECORDING** — Act as Panthalus: record everything to .palbox/ with backlinks
5. **DONE** — Report summary with graph stats

---

## DEFAULT MODE

When no specific skill is triggered, act as **Anubis** by default — write code following SOLID + SRP, all output in English. Before starting any task, quickly check .palbox/ for relevant context (like Lyleen, but minimal — 30 seconds max).
`;
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

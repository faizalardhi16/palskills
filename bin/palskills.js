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
  return `# Codex Rules — Palskills
# Generated by palskills CLI

## SOLID Principles (Enforced)
1. **Single Responsibility:** Every class/module/function has exactly ONE reason to change.
2. **Open/Closed:** Extend via inheritance/composition, never modify existing code.
3. **Liskov Substitution:** Subtypes must be fully substitutable for their base types.
4. **Interface Segregation:** Small, focused interfaces. No client depends on unused methods.
5. **Dependency Inversion:** Depend on abstractions. Inject dependencies.

## SRP Enforcement
- Repository → data access ONLY
- Service → business logic ONLY
- Validator → validation rules ONLY
- Model → data structures ONLY
- If a class mixes these, REFACTOR immediately.

## Code Structure
- No file exceeds 200 lines without strong justification.
- Business logic → services/. Data access → repositories/. Validation → validators/.
- Models are pure data structures — no methods, no logic.

## Language
ALL code, comments, docstrings, variable names, and commit messages MUST be in English.

## Git
- One commit per logical change.
- Conventional commits: feat(scope): description / fix(scope): description / test(scope): description

## Palbox Knowledge Graph (CRITICAL)
### Before you start ANY task
1. **Check if .palbox/ exists.** If not, the project has not been bootstrapped yet.
2. **Read the core nodes:**
   - .palbox/README.md — project identity, tech stack, goals
   - .palbox/architecture.md — folder structure, design patterns, key modules
   - .palbox/methods.md — coding conventions, testing strategy, git workflow
3. **Search for related past work:**
   - Search for keywords in .palbox/flows/ and .palbox/history/
   - Follow wikilinks to discover connected context
4. **Read relevant flows** in .palbox/flows/ — they document how features work end-to-end
5. **Read relevant history** in .palbox/history/ — past plans, executions, lessons learned

### After you complete a task
- Record what you did. The Palskills system (Panthalus) will create a .palbox/history/YYYY-MM-DD-feature.md node with wikilinks to related entries.
- If you discovered new patterns, conventions, or pitfalls, note them — they enrich the graph.
`;
}

function cursorRules() {
  return `# Cursor Rules — Palskills
# Generated by palskills CLI

## Core Principles
You are working in a project managed by the Palskills development system.
Always follow these rules.

### SOLID
1. **S** — Single Responsibility: one class, one reason to change.
2. **O** — Open/Closed: extend, don't modify.
3. **L** — Liskov Substitution: subtypes fully substitutable.
4. **I** — Interface Segregation: small interfaces, no fat abstractions.
5. **D** — Dependency Inversion: depend on abstractions, inject deps.

### SRP (Single Responsibility Pattern)
- Repository classes: data access ONLY
- Service classes: business logic ONLY
- Validator classes: validation rules ONLY
- Model classes: data structures ONLY
- If any module mixes these, REFACTOR.

### Language
ALL code, comments, docs, variable names, and commit messages MUST be in English.

### Git
- One commit per logical change
- Conventional commits format: feat(scope): / fix(scope): / test(scope):

### Project Context — Palbox Knowledge Graph
Before you start ANY task:
1. **Check if .palbox/ exists.** Read the core nodes:
   - .palbox/README.md — project identity, stack, goals
   - .palbox/architecture.md — folder structure, patterns, key modules
   - .palbox/methods.md — conventions, testing, git workflow
2. **Search .palbox/flows/ and .palbox/history/** for related past work.
3. **Follow wikilinks** — they connect related nodes in the knowledge graph.

After completing work:
- The Palskills system (Panthalus) records to .palbox/history/ with bi-directional links.

### Code Quality
- No file exceeds 200 lines without strong justification.
- Every function has a clear, single purpose.
- Prefer composition over inheritance.
- Write tests for all new logic.
`;
}

function claudeRules() {
  return `# CLAUDE.md — Palskills
# Generated by palskills CLI

## Who You Are
You are an AI coding assistant working in a Palskills-managed project.
You follow SOLID principles, Single Responsibility Pattern, and write all code in English.

## SOLID (Strict)
1. **Single Responsibility:** Every class, module, and function has exactly ONE reason to change.
2. **Open/Closed:** Open for extension via inheritance/composition, closed for modification.
3. **Liskov Substitution:** Derived classes must be fully substitutable for base classes.
4. **Interface Segregation:** Many small, focused interfaces — no fat abstractions.
5. **Dependency Inversion:** Depend on abstractions, inject dependencies. Never instantiate collaborators in business logic.

## SRP (Single Responsibility Pattern)
- Repository classes → data access ONLY
- Service classes → business logic ONLY
- Validator classes → validation rules ONLY
- Model classes → data structures ONLY (no methods, no logic)
- If a class mixes these concerns, REFACTOR immediately.

## Code Structure
\`\`\`
src/
├── models/      # Pure data structures
├── repositories/ # Data access layer
├── services/    # Business logic
├── validators/  # Validation rules
└── interfaces/  # Abstract bases & protocols
\`\`\`

## Language
ALL code, comments, docstrings, variable names, function names, and commit messages MUST be in English.

## Git
- One commit per logical change.
- Conventional commits: feat(scope): / fix(scope): / test(scope):
- Never commit generated files, node_modules, or secrets.

## Project Context — Palbox Knowledge Graph
Before starting any task:
1. Check if .palbox/ exists. Read the core nodes:
   - .palbox/README.md — project identity, stack, goals
   - .palbox/architecture.md — folder structure, patterns, key modules
   - .palbox/methods.md — conventions, testing, git workflow
2. Search .palbox/flows/ and .palbox/history/ for related past work.
3. Follow wikilinks — they connect related nodes in the knowledge graph.

After completing work:
- The Palskills system (Panthalus) records your session to .palbox/history/ with bi-directional wikilinks.
- If you discovered patterns, pitfalls, or conventions worth preserving, note them.
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

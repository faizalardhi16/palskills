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
  const skillNames = ['elphidran', 'astegon', 'blazamut', 'lyleen', 'jetdragon', 'anubis', 'panthalus', 'astralym', 'verdash'];

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
  const src = path.join(__dirname, '..', 'skills', skill, 'SKILL.md');
  if (!fs.existsSync(src)) return '';
  return fs.readFileSync(src, 'utf8');
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

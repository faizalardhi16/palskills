#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const os = require('os');

const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const NC = '\x1b[0m';

const hermesHome = process.env.HERMES_HOME || path.join(os.homedir(), '.hermes');
const target = path.join(hermesHome, 'skills', 'palskills');
const src = path.join(__dirname, '..', 'skills');

console.log('╔══════════════════════════════════════╗');
console.log('║     Palskills Installer             ║');
console.log('╚══════════════════════════════════════╝');
console.log('');

if (!fs.existsSync(src)) {
  console.log(`  ${YELLOW}⚠${NC}  Skills source not found. Skipping.`);
  process.exit(0);
}

fs.mkdirSync(target, { recursive: true });

const skills = fs.readdirSync(src);
for (const skill of skills) {
  const skillMd = path.join(src, skill, 'SKILL.md');
  if (fs.existsSync(skillMd)) {
    const dest = path.join(target, skill);
    fs.mkdirSync(dest, { recursive: true });
    fs.copyFileSync(skillMd, path.join(dest, 'SKILL.md'));
    console.log(`  ${GREEN}✓${NC} ${skill}`);
  }
}

console.log('');
console.log(`  ${GREEN}✅ Done!${NC} Skills installed to ${target}`);

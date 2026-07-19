---
name: elphidran
description: "Design system recommender — analyzes the project and generates a design.md with base colors, typography, spacing, and UI patterns for Anubis to apply during development."
version: 1.0.0
author: Palskills
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [palskills, design-system, ui, theming, colors, typography]
    related_skills: [astralym, anubis, panthalus]
---

# Elphidran — Design System Recommender

Elphidran is the **design architect** of the palskills system. When loaded, it analyzes the project's nature and generates a `design.md` file — a complete design system specification that **Anubis** reads when implementing any UI component. This ensures visual consistency across the entire application.

## When Elphidran Runs

- User says: "Elphidran: generate design system" or "Elphidran: analyze my app"
- Or as part of Astralym pipeline (optional DESIGN step before DEVELOPMENT)

## How It Works

### Step 1: Analyze the Project

Elphidran examines what the project IS to recommend appropriate design:

```bash
# Read project identity
cat .palbox/README.md

# Detect tech stack (React, Vue, Flutter, etc.)
cat package.json 2>/dev/null | grep -E "react|vue|angular|svelte|next"
cat pubspec.yaml 2>/dev/null  # Flutter

# Check if any UI library is already in use
cat package.json 2>/dev/null | grep -E "tailwind|mui|chakra|shadcn|antd|bootstrap"
```

### Step 2: Ask Insight Questions

Elphidran asks the user design-oriented questions to tailor the system:

1. **Vibe:** "What's the app's personality? (professional/corporate, playful/creative, minimal/clean, dark/serious)"
2. **Industry:** "What domain? (healthcare, finance, education, gaming, e-commerce, social media)"
3. **Primary color:** "Any brand color already? Or should I suggest one?"
4. **Dark mode:** "Need dark mode support? (yes / light-only / dark-first)"
5. **Audience:** "Who uses this? (internal team, public consumers, enterprise clients)"

### Step 3: Generate design.md

Based on analysis + user answers, create `.palbox/design.md`:

```markdown
# Design System
**Generated:** YYYY-MM-DD
**Architect:** Elphidran (Palskills)
**Vibe:** [chosen vibe]
**Dark Mode:** [yes/no]

## Color Palette

### Light Theme
| Token | Hex | Usage |
|-------|-----|-------|
| `primary` | #2563EB | Primary buttons, links, active states |
| `primary-hover` | #1D4ED8 | Button hover, link hover |
| `primary-light` | #DBEAFE | Selected backgrounds, badges |
| `secondary` | #7C3AED | Secondary actions, accents |
| `surface` | #FFFFFF | Card backgrounds, modals |
| `background` | #F8FAFC | Page background |
| `text-primary` | #0F172A | Headings, body text |
| `text-secondary` | #475569 | Captions, metadata |
| `border` | #E2E8F0 | Dividers, input borders |
| `success` | #16A34A | Success states, confirmations |
| `warning` | #F59E0B | Warnings, pending states |
| `error` | #DC2626 | Errors, destructive actions |
| `info` | #0EA5E9 | Info alerts, tooltips |

### Dark Theme (if enabled)
| Token | Hex |
|-------|-----|
| `surface` | #1E293B |
| `background` | #0F172A |
| `text-primary` | #F1F5F9 |
| `text-secondary` | #94A3B8 |
| `border` | #334155 |

## Typography

| Token | Font | Size | Weight | Line Height | Usage |
|-------|------|------|--------|-------------|-------|
| `text-xs` | Inter | 12px | 400 | 16px | Captions, badges |
| `text-sm` | Inter | 14px | 400 | 20px | Body small, metadata |
| `text-base` | Inter | 16px | 400 | 24px | Body text |
| `text-lg` | Inter | 18px | 500 | 28px | Subheadings |
| `text-xl` | Inter | 20px | 600 | 28px | Card titles |
| `text-2xl` | Inter | 24px | 700 | 32px | Section headers |
| `text-3xl` | Inter | 30px | 800 | 36px | Page titles |
| `text-4xl` | Inter | 36px | 800 | 40px | Hero headings |

**Font Family:** Inter (primary), system-ui (fallback)
**Monospace:** JetBrains Mono (code blocks)

## Spacing Scale

| Token | Value | Usage |
|-------|-------|-------|
| `space-0` | 0px | No spacing |
| `space-1` | 4px | Icon-text gap, tight |
| `space-2` | 8px | Inline element gap |
| `space-3` | 12px | List item gap |
| `space-4` | 16px | Standard padding, card padding |
| `space-5` | 20px | Large padding |
| `space-6` | 24px | Section gap |
| `space-8` | 32px | Section margin |
| `space-10` | 40px | Page section gap |
| `space-12` | 48px | Major section divider |
| `space-16` | 64px | Hero spacing |

## Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `radius-sm` | 4px | Inputs, small elements |
| `radius-md` | 8px | Cards, modals, buttons |
| `radius-lg` | 12px | Large cards, panels |
| `radius-full` | 9999px | Pills, badges, avatars |

## Shadows

| Token | Value | Usage |
|-------|-------|-------|
| `shadow-sm` | 0 1px 2px rgba(0,0,0,0.05) | Subtle cards |
| `shadow-md` | 0 4px 6px rgba(0,0,0,0.07) | Dropdowns, elevated cards |
| `shadow-lg` | 0 10px 15px rgba(0,0,0,0.1) | Modals |
| `shadow-xl` | 0 20px 25px rgba(0,0,0,0.15) | Drawers, slideovers |

## Component Patterns

### Buttons
- **Primary:** `bg-primary text-white rounded-md px-4 py-2 hover:bg-primary-hover`
- **Secondary:** `bg-transparent border border-primary text-primary rounded-md px-4 py-2`
- **Ghost:** `bg-transparent text-primary hover:bg-primary-light rounded-md px-4 py-2`
- **Danger:** `bg-error text-white rounded-md px-4 py-2`
- **Sizes:** sm (32px h), md (40px h), lg (48px h)

### Inputs
- **Default:** `border border-border rounded-sm bg-surface text-text-primary px-3 py-2`
- **Focus:** `border-primary ring-2 ring-primary-light outline-none`
- **Error:** `border-error ring-2 ring-red-100`
- **Disabled:** `bg-gray-50 text-text-secondary cursor-not-allowed`

### Cards
- `bg-surface rounded-md shadow-sm border border-border p-4`
- Card header: `text-lg font-semibold mb-2`
- Card body: `text-sm text-text-secondary`

### Layout
- Max content width: 1200px
- Sidebar (if applicable): 256px
- Responsive breakpoints: sm(640), md(768), lg(1024), xl(1280)

## Implementation Notes for Anubis

When implementing UI with this design system:
1. **Never hardcode colors.** Use CSS variables or design tokens.
2. **Use the spacing scale.** No arbitrary px values.
3. **All text must use typography tokens.** No raw font-size declarations.
4. **Dark mode (if enabled):** Every component must support both themes via CSS variables.
5. **Responsive first.** Mobile → tablet → desktop.
6. **Consistent patterns.** Buttons always look like buttons. Inputs always look like inputs.
```

### Step 4: Report

```
## Design System Generated

**File:** .palbox/design.md
**Palette:** [primary color] based on [vibe/industry]
**Font:** [font family]
**Dark mode:** [yes / light-only]

Anubis will now use these tokens when building UI components.
```

## Design Recommendations by Industry

Elphidran has opinionated defaults per industry:

| Industry | Primary | Vibe | Font |
|----------|---------|------|------|
| Healthcare | #0891B2 (teal) | Trustworthy, clean | Inter |
| Finance | #1E40AF (deep blue) | Professional, serious | IBM Plex Sans |
| Education | #7C3AED (violet) | Engaging, warm | Inter |
| Gaming | #DC2626 (red) | Bold, energetic | Poppins |
| E-commerce | #2563EB (blue) | Action-oriented | Inter |
| Social Media | #DB2777 (pink) | Vibrant, social | Inter |
| Developer Tools | #0F172A (slate) | Minimal, dark-first | JetBrains Mono |
| Enterprise | #0F766E (dark teal) | Corporate, reliable | Inter |
| Creative Agency | #F59E0B (amber) | Bold, artistic | Playfair Display |

## Rules

1. **Always ask questions** — don't assume colors/vibes
2. **Generate design.md** — not just verbal recommendations
3. **Use design tokens** — never raw values in the spec
4. **Anubis-aware** — every section includes implementation notes
5. **One design system per project** — update, don't duplicate
6. **Respect existing** — if the project already has a design system, study and extend it

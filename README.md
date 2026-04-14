# mdigitalcn CLI

A Rust CLI for scaffolding React/TypeScript projects and adding UI components, widgets, pages, layouts, and modules from the mdigitalcn ecosystem.

## Features

- **80+ UI Components** — Full component library built on Tailwind CSS v4
- **47 Widgets** — Auth forms, CRM cards, charts, tables, marketing sections
- **34 Pages** — Dashboard, settings, auth, ecommerce, calendar, messaging
- **17 Layouts** — Sidebar, topnav, auth, landing, docs, wizard
- **8 Modules** — Auth, CRM, ecommerce, messaging, HRM, settings
- **10 Project Templates** — Vite and Next.js starters (SaaS, dashboard, blog, docs, portfolio, commerce, landing)
- **Third-party Registries** — Add custom component registries
- **GitHub Token Support** — For private repos and higher rate limits

## Installation

### npm (recommended)
```bash
npm install -g @mdigitalcn/cli
```

### Cargo
```bash
cargo install mdigitalcn-cli
```

### Build from Source
```bash
git clone https://github.com/mdigitalcn/cli
cd cli
cargo build --release
# Binary: target/release/mdigitalcn
```

## Quick Start

### Create a Project
```bash
# Base scaffold (interactive)
mdigitalcn init

# Base scaffold (non-interactive)
mdigitalcn init --name my-app --framework vite

# Scaffold + registry template
mdigitalcn init --template vite/dashboard --name my-app
mdigitalcn init --template nextjs/saas --name my-saas --cicd github
```

When using `--template`, the CLI scaffolds a base project (configs, tooling, package.json) then overlays the template's app code from the registry.

### Browse Templates
```bash
mdigitalcn template list
mdigitalcn template info vite/dashboard
```

Available templates:

| Template | Description |
|----------|-------------|
| `vite/saas` | SaaS with auth, billing, team management |
| `vite/dashboard` | Admin panel with data tables, stats, user management |
| `vite/landing` | Marketing landing page |
| `vite/commerce` | E-commerce storefront |
| `vite/blog` | Blog with posts, tags, search |
| `vite/docs` | Documentation site with sidebar, search, TOC |
| `vite/portfolio` | Project showcase with detail pages |
| `nextjs/saas` | Next.js SaaS with SSR |
| `nextjs/landing` | Next.js landing page |
| `nextjs/commerce` | Next.js e-commerce |

### Add Components
```bash
mdigitalcn component list
mdigitalcn component add button input select table
mdigitalcn component info table
```

### Add Widgets
```bash
mdigitalcn widget list
mdigitalcn widget add login-form kpi-card hero-section
mdigitalcn widget info login-form
```

### Add Pages
```bash
mdigitalcn page list
mdigitalcn page add login-page dashboard-page billing-page
```

### Add Layouts
```bash
mdigitalcn layout list
mdigitalcn layout add sidebar-layout split-auth-layout
```

### Add Modules
```bash
mdigitalcn module list
mdigitalcn module add auth ecommerce messaging
```

### Add Features & Configs
```bash
mdigitalcn add prettier eslint docker husky
mdigitalcn add                # Interactive picker
mdigitalcn list features
```

## Registry System

Each command fetches from GitHub registries with caching (1 hour TTL):

| Command     | Registry Repo       | Description                         |
|-------------|---------------------|-------------------------------------|
| `component` | `mdigitalcn_uikit`    | UI primitives (button, input, etc.) |
| `widget`    | `mdigitalcn_widgets`  | Composed widgets (forms, cards)     |
| `page`      | `mdigitalcn_pages`    | Full page templates                 |
| `module`    | `mdigitalcn_modules`  | Business logic modules              |
| `layout`    | `mdigitalcn_layouts`  | Layout templates                    |
| `template`  | `mdigitalcn_templates`| Project starters                    |

### Third-party Registries
```bash
mdigitalcn registry add @acme github:acme/widgets
mdigitalcn registry browse @acme
mdigitalcn widget add @acme/custom-widget
mdigitalcn registry list
mdigitalcn registry remove @acme
```

### Private Repos
```bash
export MDIGITALCN_GITHUB_TOKEN=ghp_your_token
# Or per-registry auth
mdigitalcn registry add @private github:org/repo --auth
```

## Project Config (`.mdigitalcn.json`)

Created by `mdigitalcn init`, tracks installed items and custom paths:

```json
{
  "version": "1.0",
  "framework": "vite",
  "paths": {
    "components": "src/components/ui",
    "widgets": "src/widgets",
    "pages": "src/pages",
    "layouts": "src/layouts",
    "modules": "src/modules"
  },
  "generated": {
    "widgets": ["login-form", "kpi-card"],
    "layouts": ["sidebar-layout"]
  },
  "registry": {
    "owner": "mdigitalcn",
    "branch": "main"
  }
}
```

## Component Dependencies

The CLI automatically resolves internal dependencies:

```bash
$ mdigitalcn component add table
# Automatically includes: button, checkbox, input, select, pagination, popover, toggle-group
# Plus foundation files: utils.ts, types.ts, variants.ts
```

External npm dependencies are written to your `package.json` automatically. After adding, run:
```bash
npm install   # or pnpm install / yarn
```

## Commands Reference

```
mdigitalcn init                     Scaffold a new project (+ optional template overlay)
mdigitalcn component <action>       UI primitives (add, list, info, status)
mdigitalcn widget <action>          Composed widgets
mdigitalcn page <action>            Page templates
mdigitalcn module <action>          Business logic modules
mdigitalcn layout <action>          Layout templates
mdigitalcn template <action>        Browse project starters (list, info)
mdigitalcn registry <action>        Manage registries (add, remove, list, browse)
mdigitalcn add [configs...]         Add features/configs (prettier, eslint, docker, etc.)
mdigitalcn list                     Show all registry types, features, configs
```

Global flags: `-q` (quiet), `-v` (verbose)

## License

MIT

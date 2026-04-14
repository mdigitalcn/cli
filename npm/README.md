# @mdigitalcn/cli

npm wrapper for the [mdigitalcn CLI](https://github.com/mdigitalcn/cli) — scaffold React/TypeScript projects and add components, widgets, pages, and layouts from the mdigitalcn ecosystem.

## Install

```bash
npm install -g @mdigitalcn/cli
```

The postinstall script downloads the correct pre-built binary for your platform automatically.

## Usage

```bash
mdigitalcn init my-app
mdigitalcn init my-app --framework vite
mdigitalcn init my-app --template vite/dashboard

mdigitalcn component add button input table
mdigitalcn widget add login-form kpi-card
mdigitalcn page add dashboard-page
mdigitalcn layout add sidebar-layout
mdigitalcn module add auth ecommerce
```

Or run without installing:

```bash
npx @mdigitalcn/cli init my-app
```

## Supported platforms

| Platform | Architecture |
|----------|-------------|
| macOS | arm64, x86_64 (universal binary) |
| Linux | x86_64, arm64 (musl) |
| Windows | x86_64 |

## Full documentation

See [github.com/mdigitalcn/cli](https://github.com/mdigitalcn/cli) for the complete command reference and configuration docs.

## License

MIT

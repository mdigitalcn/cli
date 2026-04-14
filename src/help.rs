// ─── Clap Styles (Issue #16) ─────────────────────────────────
use clap::builder::styling::{AnsiColor, Effects, Styles};

pub const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

// ─── Main CLI ────────────────────────────────────────────────

pub const MAIN_ABOUT: &str = "mdigitalcn CLI — Frontend scaffolding and component registry";
pub const MAIN_LONG_ABOUT: &str = "\
Scaffold projects, add components, widgets, pages, layouts, and modules from the mdigitalcn registry.\n\
Manage third-party registries, add configs, and browse starter templates.";

pub const MAIN_HELP_TEMPLATE: &str = "\
{about}\n\n\
{usage-heading} {usage}\n\n\
{all-args}\n\
{after-help}";

pub const MAIN_AFTER_HELP: &str = "\
Examples:
  mdigitalcn init --name my-app --framework vite
  mdigitalcn init --template vite/dashboard --name my-app
  mdigitalcn component add button input accordion
  mdigitalcn add prettier eslint husky
  mdigitalcn component list --search date
  mdigitalcn list";

// ─── Init ────────────────────────────────────────────────────

pub const CREATE_ABOUT: &str = "Initialize a new project";
pub const CREATE_LONG_ABOUT: &str = "\
Creates a new project with your chosen framework, features, and configurations.\n\n\
Run without flags for interactive mode, or pass flags for scripted usage.\n\
Use --template to start from a registry starter (e.g., mdigitalcn init --template vite/dashboard --name my-app).";

pub const CREATE_AFTER_HELP: &str = "\
Examples:
  mdigitalcn init                                            Interactive mode
  mdigitalcn init --name my-app --framework vite             Vite + React (non-interactive)
  mdigitalcn init --name app --framework nextjs --cicd github
  mdigitalcn init --template vite/dashboard --name my-app    Start from a template
  mdigitalcn init --name app --framework vite --uikit        Include mdigitalcn Kit";

// ─── Add (features/configs) ─────────────────────────────────

pub const ADD_ABOUT: &str = "Add configs, features, or registry items to a project";
pub const ADD_LONG_ABOUT: &str = "\
Add configs (prettier, eslint), features (tanstackquery), or registry items (components, widgets, pages) to an mdigitalcn project.\n\n\
Supports plain names (auto-detected), typed prefixes (component:button), and namespaced items (@acme/card).\n\
Run with no arguments for an interactive picker.";

pub const ADD_AFTER_HELP: &str = "\
Examples:
  mdigitalcn add                          Interactive picker
  mdigitalcn add prettier eslint husky    Add configs
  mdigitalcn add tanstackquery            Add TanStack Query feature
  mdigitalcn add component:button         Add a component (explicit type)
  mdigitalcn add button hero-section      Auto-detect type from registries
  mdigitalcn add c:button w:hero          Short prefixes (c/w/p/m/l)
  mdigitalcn add @acme/card               Add from third-party registry";

// ─── List / overview ─────────────────────────────────────────

pub const LIST_ABOUT: &str = "Show project overview: registries, features, and configs";
pub const LIST_AFTER_HELP: &str = "\
Examples:
  mdigitalcn list              Show all registries and features
  mdigitalcn list features     Show available features and configs";

// ─── Template ────────────────────────────────────────────────

pub const TEMPLATE_ABOUT: &str = "Browse and inspect starter templates";
pub const TEMPLATE_AFTER_HELP: &str = "\
Examples:
  mdigitalcn template                         List all templates
  mdigitalcn template list --search dashboard Search templates
  mdigitalcn template info vite/saas          Show template details";

// ─── Registry ────────────────────────────────────────────────

pub const REGISTRY_ABOUT: &str = "Configure third-party component registries";
pub const REGISTRY_AFTER_HELP: &str = "\
Examples:
  mdigitalcn registry add @acme github:acme/components  Add a registry
  mdigitalcn registry remove @acme                      Remove a registry
  mdigitalcn registry list                              List configured registries
  mdigitalcn registry browse @acme --search button      Browse a registry";

// ─── Interactive prompts ─────────────────────────────────────

pub const PROJECT_NAME_PROMPT: &str = "Project name";
pub const TEMPLATE_PROMPT: &str = "Framework template";
pub const TANSTACK_ROUTER_PROMPT: &str = "Add TanStack Router?";
pub const CICD_PROMPT: &str = "CI/CD platform";
pub const QUERY_PROMPT: &str = "API query library";

// ─── Option descriptions ────────────────────────────────────

pub const VITE_DESC: &str = "Vite + React";
pub const NEXTJS_DESC: &str = "Next.js (SSR/SSG)";
pub const WEBVIEW_DESC: &str = "Webview (Tauri)";
pub const PWA_DESC: &str = "PWA";
pub const GITHUB_CICD_DESC: &str = "GitHub Actions";
pub const GITLAB_CICD_DESC: &str = "GitLab CI/CD";
pub const NO_CICD_DESC: &str = "Skip";
pub const RTK_QUERY_DESC: &str = "RTK Query";
pub const TANSTACK_QUERY_DESC: &str = "TanStack Query";
pub const NO_QUERY_DESC: &str = "Skip";

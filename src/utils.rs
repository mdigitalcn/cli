use std::path::Path;

pub fn detect_package_manager(project_root: &str) -> &'static str {
    let root = Path::new(project_root);
    if root.join("pnpm-lock.yaml").exists() {
        "pnpm add"
    } else if root.join("yarn.lock").exists() {
        "yarn add"
    } else if root.join("bun.lockb").exists() || root.join("bun.lock").exists() {
        "bun add"
    } else {
        "npm install"
    }
}

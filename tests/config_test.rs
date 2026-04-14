use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn scaffold_project(dir: &std::path::Path, name: &str) {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir)
        .args(["init", "--name", name, "--framework", "vite", "--cicd", "none", "--query", "none"])
        .assert()
        .success();
}

#[test]
fn add_prettier_to_project() {
    let dir = tempdir().unwrap();
    scaffold_project(dir.path(), "test_prettier");
    let project_path = dir.path().join("test_prettier");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project_path)
        .args(["add", "prettier"])
        .assert()
        .success();

    assert!(project_path.join(".prettierrc").exists(), ".prettierrc missing");
    assert!(project_path.join(".prettierignore").exists(), ".prettierignore missing");

    // Should contain jsxSingleQuote since project has react
    let content = fs::read_to_string(project_path.join(".prettierrc")).unwrap();
    assert!(content.contains("jsxSingleQuote"), "Missing jsxSingleQuote for React project");
}

#[test]
fn add_eslint_to_project() {
    let dir = tempdir().unwrap();
    scaffold_project(dir.path(), "test_eslint");
    let project_path = dir.path().join("test_eslint");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project_path)
        .args(["add", "eslint"])
        .assert()
        .success();

    assert!(project_path.join("eslint.config.js").exists(), "eslint.config.js missing");
}

#[test]
fn add_husky_creates_nested_dirs() {
    let dir = tempdir().unwrap();
    scaffold_project(dir.path(), "test_husky");
    let project_path = dir.path().join("test_husky");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project_path)
        .args(["add", "husky"])
        .assert()
        .success();

    assert!(project_path.join(".husky/pre-commit").exists(), ".husky/pre-commit missing");
    assert!(project_path.join(".lintstagedrc").exists(), ".lintstagedrc missing");
}

#[test]
fn add_tsconfig_to_project() {
    let dir = tempdir().unwrap();
    scaffold_project(dir.path(), "test_tsconfig");
    let project_path = dir.path().join("test_tsconfig");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project_path)
        .args(["add", "tsconfig"])
        .assert()
        .success();

    assert!(project_path.join("tsconfig.json").exists(), "tsconfig.json missing");
    let content = fs::read_to_string(project_path.join("tsconfig.json")).unwrap();
    assert!(content.contains("@/*"), "Missing @/* alias for FSD");
}

#[test]
fn add_githubcicd_to_project() {
    let dir = tempdir().unwrap();
    scaffold_project(dir.path(), "test_ghcicd");
    let project_path = dir.path().join("test_ghcicd");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project_path)
        .args(["add", "githubcicd"])
        .assert()
        .success();

    assert!(
        project_path.join(".github/workflows/ci-cd.yml").exists(),
        ".github/workflows/ci-cd.yml missing"
    );
}

#[test]
fn add_dockerfile_to_project() {
    let dir = tempdir().unwrap();
    scaffold_project(dir.path(), "test_docker");
    let project_path = dir.path().join("test_docker");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project_path)
        .args(["add", "dockerfile"])
        .assert()
        .success();

    assert!(project_path.join("Dockerfile").exists(), "Dockerfile missing");
}

#[test]
fn add_multiple_configs() {
    let dir = tempdir().unwrap();
    scaffold_project(dir.path(), "test_multi");
    let project_path = dir.path().join("test_multi");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project_path)
        .args(["add", "prettier", "eslint", "commitlint"])
        .assert()
        .success();

    assert!(project_path.join(".prettierrc").exists());
    assert!(project_path.join("eslint.config.js").exists());
    assert!(project_path.join("commitlint.config.js").exists());
}

#[test]
fn add_unknown_config_fails() {
    let dir = tempdir().unwrap();
    scaffold_project(dir.path(), "test_unknown");
    let project_path = dir.path().join("test_unknown");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project_path)
        .args(["add", "nonexistent"])
        .assert()
        .failure();
}

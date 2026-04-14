use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn scaffold_vite_creates_project() {
    let dir = tempdir().unwrap();
    let project_name = "test_vite_project";
    let project_path = dir.path().join(project_name);

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "vite", "--cicd", "none", "--query", "none"])
        .assert()
        .success();

    assert!(project_path.join("package.json").exists(), "package.json missing");
    assert!(project_path.join("vite.config.ts").exists(), "vite.config.ts missing");
    assert!(project_path.join("tsconfig.json").exists(), "tsconfig.json missing");
    assert!(project_path.join("src").exists(), "src/ missing");
    assert!(project_path.join(".mdigitalcn.json").exists(), ".mdigitalcn.json missing");
    assert!(!project_path.join("next.config.ts").exists(), "next.config.ts should not exist for vite");
}

#[test]
fn scaffold_nextjs_creates_project() {
    let dir = tempdir().unwrap();
    let project_name = "test_nextjs_project";
    let project_path = dir.path().join(project_name);

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "nextjs", "--cicd", "none", "--query", "none"])
        .assert()
        .success();

    assert!(project_path.join("package.json").exists(), "package.json missing");
    assert!(project_path.join("next.config.ts").exists(), "next.config.ts missing");
    assert!(project_path.join("src").exists(), "src/ missing");
    assert!(project_path.join(".mdigitalcn.json").exists(), ".mdigitalcn.json missing");
}

#[test]
fn scaffold_with_github_cicd() {
    let dir = tempdir().unwrap();
    let project_name = "test_cicd_project";
    let project_path = dir.path().join(project_name);

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "vite", "--cicd", "github", "--query", "none"])
        .assert()
        .success();

    assert!(
        project_path.join(".github/workflows/ci-cd.yml").exists(),
        "GitHub CI/CD workflow missing"
    );
}

#[test]
fn scaffold_without_cicd() {
    let dir = tempdir().unwrap();
    let project_name = "test_no_cicd";
    let project_path = dir.path().join(project_name);

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "vite", "--cicd", "none", "--query", "none"])
        .assert()
        .success();

    assert!(!project_path.join(".github").exists(), "CI/CD dir should not exist");
    assert!(!project_path.join(".gitlab-ci.yml").exists(), "GitLab CI should not exist");
}

#[test]
fn scaffold_force_overwrites() {
    let dir = tempdir().unwrap();
    let project_name = "test_force";
    let project_path = dir.path().join(project_name);

    // Create first
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "vite", "--cicd", "none", "--query", "none"])
        .assert()
        .success();

    // Create again with --force
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "vite", "--cicd", "none", "--query", "none", "--force"])
        .assert()
        .success();

    assert!(project_path.join("package.json").exists());
}

#[test]
fn scaffold_rejects_existing_without_force() {
    let dir = tempdir().unwrap();
    let project_name = "test_no_force";

    // Create first
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "vite", "--cicd", "none", "--query", "none"])
        .assert()
        .success();

    // Try again without --force
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "vite", "--cicd", "none", "--query", "none"])
        .assert()
        .failure();
}

#[test]
fn scaffold_rejects_invalid_name() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "node_modules", "--framework", "vite"])
        .assert()
        .failure();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "../escape", "--framework", "vite"])
        .assert()
        .failure();
}

#[test]
fn scaffold_mdigitalcn_json_has_framework() {
    let dir = tempdir().unwrap();
    let project_name = "test_config";
    let project_path = dir.path().join(project_name);

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", project_name, "--framework", "vite", "--cicd", "none", "--query", "none"])
        .assert()
        .success();

    let config_content = std::fs::read_to_string(project_path.join(".mdigitalcn.json")).unwrap();
    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert_eq!(config["framework"], "vite");
    assert_eq!(config["version"], "1.0");
}

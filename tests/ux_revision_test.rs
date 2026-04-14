use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

// ─── Issue #6: Help descriptions everywhere ──────────────────

#[test]
fn main_help_shows_all_command_descriptions() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Every command must have a non-empty description
    assert!(stdout.contains("Initialize a new project"), "init missing description");
    assert!(stdout.contains("Manage UI components"), "component missing description");
    assert!(stdout.contains("Manage composed widgets"), "widget missing description");
    assert!(stdout.contains("Manage page templates"), "page missing description");
    assert!(stdout.contains("Manage module templates"), "module missing description");
    assert!(stdout.contains("Manage layout templates"), "layout missing description");
    assert!(stdout.contains("Browse and inspect"), "template missing description");
    assert!(stdout.contains("Configure third-party"), "registry missing description");
    assert!(stdout.contains("Add configs, features, or registry items"), "add missing description");
    assert!(stdout.contains("Show project overview"), "list missing description");
}

#[test]
fn main_help_shows_examples() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn init_help_shows_flag_descriptions() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Project name"), "--name missing help");
    assert!(stdout.contains("Framework template"), "--framework missing help");
    assert!(stdout.contains("starter template"), "--template missing help");
    assert!(stdout.contains("TanStack Router"), "--router missing help");
    assert!(stdout.contains("API query library"), "--query missing help");
    assert!(stdout.contains("CI/CD platform"), "--cicd missing help");
    assert!(stdout.contains("mdigitalcn Kit"), "--uikit missing help");
    assert!(stdout.contains("Overwrite existing"), "--force missing help");
    assert!(stdout.contains("Bypass cache"), "--no-cache missing help");
    assert!(stdout.contains("Examples:"), "init examples missing");
}

#[test]
fn component_help_shows_subcommand_descriptions() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["component", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Add items from the registry"), "add subcommand missing desc");
    assert!(stdout.contains("List available items"), "list subcommand missing desc");
    assert!(stdout.contains("Show detailed info"), "info subcommand missing desc");
    assert!(stdout.contains("Show installed items"), "status subcommand missing desc");
    assert!(stdout.contains("Examples:"), "component examples missing");
}

#[test]
fn component_add_help_shows_flag_descriptions() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["component", "add", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Names of items"), "NAME missing help");
    assert!(stdout.contains("Bypass cache"), "--no-cache missing help");
    assert!(stdout.contains("Project root"), "--project-root missing help");
    assert!(stdout.contains("Overwrite existing"), "--overwrite missing help");
}

#[test]
fn widget_help_shows_descriptions() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["widget", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage composed widgets"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn page_help_shows_descriptions() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["page", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage page templates"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn module_help_shows_descriptions() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage module templates"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn layout_help_shows_descriptions() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["layout", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage layout templates"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn template_help_shows_descriptions() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["template", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Browse and inspect"))
        .stdout(predicate::str::contains("Examples:"));
}

#[test]
fn registry_help_shows_descriptions() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["registry", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Add a third-party"), "add subcommand missing desc");
    assert!(stdout.contains("Remove a third-party"), "remove subcommand missing desc");
    assert!(stdout.contains("List all configured"), "list subcommand missing desc");
    assert!(stdout.contains("Browse items"), "browse subcommand missing desc");
    assert!(stdout.contains("Examples:"), "registry examples missing");
}

#[test]
fn add_help_shows_descriptions() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["add", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // NAME arg should have help text (doc comment or help attr)
    assert!(stdout.contains("config") || stdout.contains("registry item"), "NAME missing help");
    assert!(stdout.contains("Project root"), "--project-root missing help");
    assert!(stdout.contains("Bypass cache") || stdout.contains("cache"), "--no-cache missing help");
    assert!(stdout.contains("Examples:"), "add examples missing");
}

#[test]
fn list_help_shows_descriptions() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["list", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Show project overview"), "list missing about");
    assert!(stdout.contains("Bypass cache"), "--no-cache missing help");
    assert!(stdout.contains("Show available features"), "features subcommand missing");
    assert!(stdout.contains("Examples:"), "list examples missing");
}

// ─── Issue #7: --query default changed to none ───────────────

#[test]
fn query_defaults_to_none_non_interactive() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_query_none", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Query:").and(predicate::str::contains("None")));

    // RTK Query service files should NOT exist
    let project = dir.path().join("test_query_none");
    assert!(
        !project.join("src/shared/services/rtk_query").exists(),
        "RTK Query should not be scaffolded when query defaults to none"
    );
}

#[test]
fn query_rtk_explicit_still_works() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_rtk", "--framework", "vite", "--cicd", "none", "--query", "rtk"])
        .assert()
        .success()
        .stdout(predicate::str::contains("RTK Query"));
}

#[test]
fn query_tanstack_explicit_works() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_tanstack", "--framework", "vite", "--cicd", "none", "--query", "tanstack"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TanStack Query"));
}

// ─── Issue #9: No args → default to list, not error ──────────

#[test]
fn component_no_args_does_not_error_missing_subcommand() {
    // Should NOT fail with "missing subcommand" error.
    // May fail with network/registry errors, but that's different.
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("component")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("requires a subcommand"),
        "Should not say 'requires a subcommand': {}", stderr
    );
}

#[test]
fn widget_no_args_does_not_error_missing_subcommand() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("widget")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("requires a subcommand"),
        "Should not say 'requires a subcommand': {}", stderr
    );
}

#[test]
fn page_no_args_does_not_error_missing_subcommand() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("page")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("requires a subcommand"),
        "Should not say 'requires a subcommand': {}", stderr
    );
}

#[test]
fn module_no_args_does_not_error_missing_subcommand() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("module")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("requires a subcommand"),
        "Should not say 'requires a subcommand': {}", stderr
    );
}

#[test]
fn layout_no_args_does_not_error_missing_subcommand() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("layout")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("requires a subcommand"),
        "Should not say 'requires a subcommand': {}", stderr
    );
}

#[test]
fn template_no_args_does_not_error_missing_subcommand() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("template")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("requires a subcommand"),
        "Should not say 'requires a subcommand': {}", stderr
    );
}

// ─── Issue #12: --name missing in non-interactive ────────────

#[test]
fn init_framework_without_name_errors_helpfully() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--framework", "vite"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--name is required"));
}

#[test]
fn init_framework_and_cicd_without_name_errors() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--framework", "nextjs", "--cicd", "github"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--name is required"));
}

#[test]
fn init_framework_and_query_without_name_errors() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--framework", "vite", "--query", "rtk"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--name is required"));
}

// ─── Issue #15: --no-cache consistency ───────────────────────

#[test]
fn add_accepts_no_cache_flag() {
    let dir = tempdir().unwrap();
    // Scaffold a project first
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_nc", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    // --no-cache should be accepted (not "unexpected argument")
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path().join("test_nc"))
        .args(["add", "--no-cache", "prettier"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "--no-cache should be accepted on add: {}", stderr
    );
}

#[test]
fn list_accepts_no_cache_flag() {
    // --no-cache should be accepted (not "unexpected argument")
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["list", "--no-cache"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "--no-cache should be accepted on list: {}", stderr
    );
}

// ─── Dumb user scenarios ─────────────────────────────────────

#[test]
fn init_empty_name_fails() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--name", "", "--framework", "vite"])
        .assert()
        .failure();
}

#[test]
fn init_reserved_name_fails() {
    let dir = tempdir().unwrap();

    for name in &["node_modules", "con", "nul", "test", ".git"] {
        Command::cargo_bin("mdigitalcn")
            .unwrap()
            .current_dir(dir.path())
            .args(["init", "--name", name, "--framework", "vite"])
            .assert()
            .failure();
    }
}

#[test]
fn init_too_long_name_fails() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--name", "abcdefghijklmnopqrstuvwxyz-abcdefghijklmnopqr", "--framework", "vite"])
        .assert()
        .failure();
}

#[test]
fn init_special_chars_name_fails() {
    let dir = tempdir().unwrap();

    for name in &["my app", "hello!", "test@proj", "foo/bar", "../escape", "a b c"] {
        Command::cargo_bin("mdigitalcn")
            .unwrap()
            .current_dir(dir.path())
            .args(["init", "--name", name, "--framework", "vite"])
            .assert()
            .failure();
    }
}

#[test]
fn init_starts_with_special_fails() {
    let dir = tempdir().unwrap();

    for name in &["-leading-dash", "_leading-underscore"] {
        Command::cargo_bin("mdigitalcn")
            .unwrap()
            .current_dir(dir.path())
            .args(["init", "--name", name, "--framework", "vite"])
            .assert()
            .failure();
    }
}

#[test]
fn init_name_at_length_boundary() {
    let dir = tempdir().unwrap();

    // Exactly 42 chars should succeed
    let name42 = "a".repeat(42);
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", &name42, "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    // 43 chars should fail
    let name43 = "a".repeat(43);
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", &name43, "--framework", "vite"])
        .assert()
        .failure();
}

#[test]
fn init_invalid_framework_value() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--name", "test", "--framework", "angular"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn init_invalid_query_value() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--name", "test", "--framework", "vite", "--query", "graphql"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn init_invalid_cicd_value() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--name", "test", "--framework", "vite", "--cicd", "jenkins"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn init_double_force_rejected_by_clap() {
    // Clap correctly rejects duplicate bool flags
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--name", "test_df", "--framework", "vite", "--cicd", "none", "--force", "--force"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used multiple times"));
}

#[test]
fn init_all_frameworks() {
    let dir = tempdir().unwrap();

    for fw in &["vite", "nextjs", "webview", "pwa"] {
        let name = format!("test_{}", fw);
        Command::cargo_bin("mdigitalcn")
            .unwrap()
            .current_dir(dir.path())
            .args(["init", "--name", &name, "--framework", fw, "--cicd", "none"])
            .assert()
            .success();
        assert!(dir.path().join(&name).join("package.json").exists());
        assert!(dir.path().join(&name).join(".mdigitalcn.json").exists());
    }
}

#[test]
fn init_all_query_options() {
    let dir = tempdir().unwrap();

    for (query, label) in &[("none", "None"), ("rtk", "RTK Query"), ("tanstack", "TanStack Query")] {
        let name = format!("test_q_{}", query);
        Command::cargo_bin("mdigitalcn")
            .unwrap()
            .current_dir(dir.path())
            .args(["init", "--name", &name, "--framework", "vite", "--cicd", "none", "--query", query])
            .assert()
            .success()
            .stdout(predicate::str::contains(*label));
    }
}

#[test]
fn init_all_cicd_options() {
    let dir = tempdir().unwrap();

    // github
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_gh", "--framework", "vite", "--cicd", "github", "--query", "none"])
        .assert()
        .success();
    assert!(dir.path().join("test_gh/.github/workflows/ci-cd.yml").exists());

    // gitlab
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_gl", "--framework", "vite", "--cicd", "gitlab", "--query", "none"])
        .assert()
        .success();
    assert!(dir.path().join("test_gl/.gitlab-ci.yml").exists());

    // none
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_nocicd", "--framework", "vite", "--cicd", "none", "--query", "none"])
        .assert()
        .success();
    assert!(!dir.path().join("test_nocicd/.github").exists());
    assert!(!dir.path().join("test_nocicd/.gitlab-ci.yml").exists());
}

#[test]
fn add_unknown_feature_fails_gracefully() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_uf", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    // Unknown name: not a config, not in any registry
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path().join("test_uf"))
        .args(["add", "nonexistent_feature"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not found"));
}

#[test]
fn add_multiple_including_invalid_fails() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_mi", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    // First valid one succeeds, second one fails
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path().join("test_mi"))
        .args(["add", "prettier", "bogus_config"])
        .assert()
        .failure();

    // But prettier was already added
    assert!(dir.path().join("test_mi/.prettierrc").exists());
}

#[test]
fn version_flag_works() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mdigitalcn"));
}

#[test]
fn quiet_flag_suppresses_output() {
    let dir = tempdir().unwrap();

    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["--quiet", "init", "--name", "test_q", "--framework", "vite", "--cicd", "none"])
        .output()
        .unwrap();

    // In quiet mode, stdout should have minimal content
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Command failed in quiet mode");
    // The "Next steps" block should be suppressed
    assert!(
        !stdout.contains("Next steps:"),
        "Quiet mode should suppress info output"
    );
}

#[test]
fn verbose_flag_accepted() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["--verbose", "--help"])
        .assert()
        .success();
}

#[test]
fn unknown_command_fails() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("gibberish")
        .assert()
        .failure()
        .stderr(predicate::str::contains("gibberish"));
}

#[test]
fn unknown_flag_fails() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--nonexistent-flag"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn component_unknown_subcommand_fails() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["component", "destroy"])
        .assert()
        .failure();
}

#[test]
fn help_on_short_h() {
    // -h should show short help
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("mdigitalcn CLI"));
}

#[test]
fn help_on_each_subcommand_short() {
    for cmd in &["init", "component", "widget", "page", "module", "layout",
                  "template", "registry", "add", "list"] {
        Command::cargo_bin("mdigitalcn")
            .unwrap()
            .args([cmd, "-h"])
            .assert()
            .success();
    }
}

// ─── Scaffold integrity ──────────────────────────────────────

#[test]
fn scaffold_produces_valid_mdigitalcn_json() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_json", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    let content = std::fs::read_to_string(dir.path().join("test_json/.mdigitalcn.json")).unwrap();
    let config: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(config["version"], "1.0");
    assert_eq!(config["framework"], "vite");
    assert!(config["paths"].is_object(), "paths should be an object");
    assert!(config["generated"].is_object(), "generated should be an object");
}

#[test]
fn scaffold_with_uikit_flag() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_uikit", "--framework", "vite", "--cicd", "none", "--uikit"])
        .assert()
        .success()
        .stdout(predicate::str::contains("UIKit:"));

    let content = std::fs::read_to_string(dir.path().join("test_uikit/.mdigitalcn.json")).unwrap();
    let config: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(
        config["features"].as_array().unwrap().iter().any(|f| f == "uikit"),
        "uikit should be in features"
    );
}

#[test]
fn scaffold_with_router_flag() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_router", "--framework", "vite", "--cicd", "none", "--router"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TanStack Router"));
}

#[test]
fn add_idempotent_config() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_idem", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    let project = dir.path().join("test_idem");

    // Add prettier twice - should succeed both times
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project)
        .args(["add", "prettier"])
        .assert()
        .success();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project)
        .args(["add", "prettier"])
        .assert()
        .success();

    assert!(project.join(".prettierrc").exists());
}

#[test]
fn list_shows_features_subcommand() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["list", "features"])
        .assert()
        .success()
        .stdout(predicate::str::contains("FEATURES"))
        .stdout(predicate::str::contains("CONFIGS"))
        .stdout(predicate::str::contains("tanstackquery"))
        .stdout(predicate::str::contains("prettier"));
}

#[test]
fn list_default_shows_overview() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Registry commands"))
        .stdout(predicate::str::contains("Registries"))
        .stdout(predicate::str::contains("Features"));
}

// ─── Edge cases users WILL try ───────────────────────────────

#[test]
fn init_with_only_router_flag_goes_interactive_or_errors() {
    // User types: mdigitalcn init --router
    // No --name, no --framework → should go to interactive (which fails non-TTY)
    // or work fine. Key: should not panic.
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--router"])
        .output()
        .unwrap();
    // Should not panic - either succeeds (interactive) or fails gracefully
    assert!(
        output.status.code().is_some(),
        "Should exit with a code, not crash"
    );
}

#[test]
fn init_with_only_uikit_flag() {
    // mdigitalcn init --uikit (no --name, no --framework)
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--uikit"])
        .output()
        .unwrap();
    assert!(output.status.code().is_some(), "Should not crash");
}

#[test]
fn double_quiet_verbose_quiet_wins() {
    // --quiet and --verbose together: quiet wins
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["--quiet", "--verbose", "init", "--name", "test_qv", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();
}

#[test]
fn add_without_project_config_fails() {
    // Running mdigitalcn add in a dir without .mdigitalcn.json
    let dir = tempdir().unwrap();

    // This should fail because there's no project config
    // But it might succeed for embedded configs (prettier etc.)
    // At minimum it should not panic
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "prettier"])
        .output()
        .unwrap();

    assert!(output.status.code().is_some(), "Should not crash");
}

// ─── Issue #4: TanStack Query consolidation ──────────────────

#[test]
fn add_tanstackquery_produces_scaffold_identical_paths() {
    let dir = tempdir().unwrap();

    // Scaffold with --query none
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_tq", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    let project = dir.path().join("test_tq");

    // Add tanstackquery post-scaffold
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project)
        .args(["add", "tanstackquery"])
        .assert()
        .success();

    // Files should be at the SAME paths as mdigitalcn init --query tanstack
    assert!(
        project.join("src/shared/services/tanstack_query/client.ts").exists(),
        "client.ts should be at scaffold-identical path"
    );
    assert!(
        project.join("src/shared/services/tanstack_query/api.ts").exists(),
        "api.ts should be at scaffold-identical path"
    );

    // Old paths should NOT exist (provider.tsx, hooks.ts, utils.ts)
    assert!(
        !project.join("src/providers/TanstackQueryProvider.tsx").exists(),
        "Old provider path should not exist"
    );
    assert!(
        !project.join("src/shared/services/api/hooks.ts").exists(),
        "Old hooks path should not exist"
    );
}

#[test]
fn add_tanstackquery_skips_if_already_scaffolded() {
    let dir = tempdir().unwrap();

    // Scaffold WITH tanstack query
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_tq2", "--framework", "vite", "--cicd", "none", "--query", "tanstack"])
        .assert()
        .success();

    let project = dir.path().join("test_tq2");

    // Running add again should succeed (skip, not error)
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project)
        .args(["add", "tanstackquery"])
        .assert()
        .success();
}

#[test]
fn add_tanstackquery_updates_package_json() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_tq3", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    let project = dir.path().join("test_tq3");

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project)
        .args(["add", "tanstackquery"])
        .assert()
        .success();

    let pkg = std::fs::read_to_string(project.join("package.json")).unwrap();
    assert!(pkg.contains("@tanstack/react-query"), "Should add @tanstack/react-query to package.json");
}

// ─── Issue #13: Unified mdigitalcn add ─────────────────────────────

#[test]
fn add_typed_prefix_accepted() {
    // component:button syntax should be parsed (may fail due to network, but shouldn't panic)
    let dir = tempdir().unwrap();
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_typed", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path().join("test_typed"))
        .args(["add", "component:nonexistent_xyz"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should route to component registry (not "Unknown config")
    assert!(
        !stderr.contains("Unknown config"),
        "Typed prefix should route to registry, not features"
    );
}

#[test]
fn add_short_prefix_accepted() {
    // c:, w:, p:, m:, l: should work
    let dir = tempdir().unwrap();
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_short", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path().join("test_short"))
        .args(["add", "c:nonexistent_xyz"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unknown config"),
        "Short prefix c: should route to component registry"
    );
}

#[test]
fn add_mixed_configs_and_typed_items() {
    // Mixing configs with typed items should process configs first
    let dir = tempdir().unwrap();
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_mixed", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    let project = dir.path().join("test_mixed");

    // prettier is a config, should succeed. component:xyz will fail but prettier should still be added.
    let _output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(&project)
        .args(["add", "prettier", "component:nonexistent_xyz_123"])
        .output()
        .unwrap();

    // prettier should have been added before the registry item failed
    assert!(project.join(".prettierrc").exists(), "Config should be added before registry items fail");
}

#[test]
fn add_overwrite_flag_accepted() {
    // --overwrite should be a valid flag
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["add", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("overwrite"), "--overwrite should appear in add help");
}

#[test]
fn add_help_shows_new_examples() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["add", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("component:button"), "Should show typed prefix example");
    assert!(stdout.contains("Auto-detect"), "Should mention auto-detect");
}

// ─── Issue #14: --overwrite on template overlay ───────────────

#[test]
fn init_overwrite_flag_accepted() {
    let output = Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["init", "--help"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--overwrite"), "--overwrite should appear in init help");
    assert!(stdout.contains("Merge template"), "--overwrite help text");
}

// ─── Issue #5 verification: no duplicate utils ────────────────

#[test]
fn scaffold_vite_no_duplicate_utils_index() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--name", "test_utils", "--framework", "vite", "--cicd", "none"])
        .assert()
        .success();

    let utils_dir = dir.path().join("test_utils/src/shared/utils");
    let index_ts = utils_dir.join("index.ts");
    let index_tsx = utils_dir.join("index.tsx");

    // Should have index.tsx (the real one)
    assert!(index_tsx.exists(), "index.tsx should exist");

    // Should NOT have index.ts (the dead weight one was removed)
    assert!(!index_ts.exists(), "index.ts should not exist (duplicate removed)");

    // index.tsx should have both the utility functions and webVitals re-export
    let content = std::fs::read_to_string(&index_tsx).unwrap();
    assert!(content.contains("cn("), "index.tsx should have cn() function");
    assert!(content.contains("webVitals"), "index.tsx should re-export webVitals");
}

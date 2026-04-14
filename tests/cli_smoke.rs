use assert_cmd::Command;

/// The binary should print help and exit 0 with --help
#[test]
fn help_flag_works() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("mdigitalcn"));
}

/// The binary should print version and exit 0 with --version
#[test]
fn version_flag_works() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("mdigitalcn"));
}

/// Page subcommand help should work
#[test]
fn page_help_works() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["page", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("page"));
}

/// Module subcommand help should work
#[test]
fn module_help_works() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["module", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("module"));
}

/// Layout subcommand help should work
#[test]
fn layout_help_works() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["layout", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("layout"));
}

/// Component subcommand help should work
#[test]
fn component_help_works() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["component", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("component"));
}

/// Widget subcommand help should work
#[test]
fn widget_help_works() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .args(["widget", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("widget"));
}

/// Running without arguments should show help (or error)
#[test]
fn no_args_shows_help() {
    Command::cargo_bin("mdigitalcn")
        .unwrap()
        .assert()
        .failure(); // clap exits with error when no subcommand given
}

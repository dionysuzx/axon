use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_refactor_dry_run_no_changes() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("foo.bar.specs.initial.v1.md"),
        "",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_axon"))
        .args([
            "refactor",
            "--from",
            "{repo}.{feature}.{type}.{variant}.v{N}",
            "--to",
            "{repo}.{type}.{feature}.{variant}.v{N}",
            "--dry-run",
            "--no-git",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(tmp.path().join("foo.bar.specs.initial.v1.md").exists());
    assert!(!tmp
        .path()
        .join("foo.specs.bar.initial.v1.md")
        .exists());
}

#[test]
fn test_refactor_no_match_exit_code() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("foo.bar.specs.initial.v1.md"),
        "",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_axon"))
        .args([
            "refactor",
            "--from",
            "{foo}.{bar}.v{N}",
            "--to",
            "{bar}.{foo}.v{N}",
            "--yes",
            "--no-git",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_refactor_placeholder_mismatch_exit_code() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("foo.bar.specs.initial.v1.md"),
        "",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_axon"))
        .args([
            "refactor",
            "--from",
            "{repo}.{feature}",
            "--to",
            "{repo}.{type}",
            "--yes",
            "--no-git",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_refactor_existing_target_exit_code() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("foo.a.specs.initial.v1.md"), "").unwrap();
    std::fs::write(tmp.path().join("foo.specs.a.initial.v1.md"), "").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_axon"))
        .args([
            "refactor",
            "--from",
            "{repo}.{feature}.{type}.{variant}.v{N}",
            "--to",
            "{repo}.{type}.{feature}.{variant}.v{N}",
            "--yes",
            "--no-git",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(tmp.path().join("foo.a.specs.initial.v1.md").exists());
    assert!(tmp.path().join("foo.specs.a.initial.v1.md").exists());
}

#[test]
fn test_refactor_empty_dir_exit_code() {
    let tmp = TempDir::new().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_axon"))
        .args([
            "refactor",
            "--from",
            "{repo}.{feature}.{type}.{variant}.v{N}",
            "--to",
            "{repo}.{type}.{feature}.{variant}.v{N}",
            "--yes",
            "--no-git",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No markdown files found"));
}

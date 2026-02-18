use std::fs;

use axon::config::{glob_match, load_config};
use axon::notes::create_daily;
use tempfile::TempDir;

#[test]
fn test_glob_match_star_suffix() {
    assert!(glob_match("daily.*", "daily.2026.02.17.md"));
}

#[test]
fn test_glob_match_star_prefix() {
    assert!(glob_match("*.md", "daily.2026.02.17.md"));
}

#[test]
fn test_glob_match_star_middle() {
    assert!(glob_match("daily.*.md", "daily.2026.02.17.md"));
}

#[test]
fn test_glob_match_exact() {
    assert!(glob_match("daily.md", "daily.md"));
    assert!(!glob_match("daily.md", "daily.txt"));
}

#[test]
fn test_glob_match_no_match() {
    assert!(!glob_match("weekly.*", "daily.2026.02.17.md"));
}

#[test]
fn test_load_config_missing_file() {
    let tmp = TempDir::new().unwrap();
    let cfg = load_config(tmp.path());
    assert!(cfg.schemas.is_empty());
}

#[test]
fn test_load_config_valid() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("axon.toml"),
        r#"
[schemas]
"daily.*" = "schema.daily"
"#,
    )
    .unwrap();

    let cfg = load_config(tmp.path());
    assert_eq!(cfg.schemas.get("daily.*").unwrap(), "schema.daily");
}

#[test]
fn test_resolve_schema_with_template() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("axon.toml"),
        r#"
[schemas]
"daily.*" = "schema.daily"
"#,
    )
    .unwrap();
    fs::write(tmp.path().join("schema.daily"), "# Daily\n\n## Log\n").unwrap();

    let cfg = load_config(tmp.path());
    let content = cfg.resolve_schema(tmp.path(), "daily.2026.02.17.md");
    assert_eq!(content.unwrap(), "# Daily\n\n## Log\n");
}

#[test]
fn test_resolve_schema_missing_schema_file() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("axon.toml"),
        r#"
[schemas]
"daily.*" = "schema.daily"
"#,
    )
    .unwrap();

    let cfg = load_config(tmp.path());
    let content = cfg.resolve_schema(tmp.path(), "daily.2026.02.17.md");
    assert!(content.is_none());
}

#[test]
fn test_create_daily_new_file_with_schema() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("axon.toml"),
        r#"
[schemas]
"daily.*" = "schema.daily"
"#,
    )
    .unwrap();
    fs::write(tmp.path().join("schema.daily"), "# Daily\n").unwrap();

    let path = create_daily(tmp.path()).unwrap();
    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "# Daily\n");
    assert!(path.file_name().unwrap().to_str().unwrap().starts_with("daily."));
    assert!(path.file_name().unwrap().to_str().unwrap().ends_with(".md"));
}

#[test]
fn test_create_daily_no_overwrite() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("axon.toml"),
        r#"
[schemas]
"daily.*" = "schema.daily"
"#,
    )
    .unwrap();
    fs::write(tmp.path().join("schema.daily"), "# Fresh\n").unwrap();

    // Create the daily file
    let path = create_daily(tmp.path()).unwrap();
    // Modify it
    fs::write(&path, "# My edits\n").unwrap();
    // Create again — should not overwrite
    let path2 = create_daily(tmp.path()).unwrap();
    assert_eq!(path, path2);
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "# My edits\n");
}

#[test]
fn test_create_daily_no_config() {
    let tmp = TempDir::new().unwrap();
    let path = create_daily(tmp.path()).unwrap();
    assert!(path.exists());
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "");
}

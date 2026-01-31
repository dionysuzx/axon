use axon::refactor::{
    check_existing_targets, check_for_duplicates, match_files, parse_refactor_pattern,
    refactor_filename, validate_placeholder_match,
};

#[test]
fn test_refactor_pattern() {
    let result = refactor_filename(
        "forkcast.objections.specs.initial.v1.md",
        "{repo}.{feature}.{type}.{variant}.v{N}",
        "{repo}.{type}.{feature}.{variant}.v{N}",
    );
    assert_eq!(
        result.unwrap(),
        "forkcast.specs.objections.initial.v1.md"
    );
}

#[test]
fn test_refactor_pattern_invalid_syntax() {
    let result = parse_refactor_pattern("invalid[pattern");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unclosed bracket"));
}

#[test]
fn test_refactor_pattern_unclosed_placeholder() {
    let result = parse_refactor_pattern("{repo}.{feature");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unclosed placeholder"));
}

#[test]
fn test_refactor_pattern_empty_placeholder() {
    let result = parse_refactor_pattern("{}.{feature}.v{N}");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Empty placeholder"));
}

#[test]
fn test_placeholder_mismatch_missing_in_target() {
    let result = validate_placeholder_match(
        "{repo}.{feature}.{type}.v{N}",
        "{repo}.{type}.v{N}",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Missing in target: {feature}"));
}

#[test]
fn test_placeholder_mismatch_missing_in_source() {
    let result = validate_placeholder_match("{repo}.{feature}.v{N}", "{repo}.{feature}.{type}.v{N}");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Missing in source: {type}"));
}

#[test]
fn test_placeholder_match_valid() {
    let result = validate_placeholder_match(
        "{repo}.{feature}.{type}.{variant}.v{N}",
        "{repo}.{type}.{feature}.{variant}.v{N}",
    );
    assert!(result.is_ok());
}

#[test]
fn test_detect_duplicate_targets() {
    let files = vec![
        "forkcast.objections.specs.initial.v1.md",
        "forkcast.objections.specs.inline.v1.md",
    ];
    let result = check_for_duplicates(
        &files,
        "{repo}.{feature}.{type}.{variant}.v{N}",
        "{repo}.{type}.v{N}",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("forkcast.specs.v1.md"));
    assert!(err.contains("would be created by"));
}

#[test]
fn test_no_duplicates_unique_targets() {
    let files = vec![
        "forkcast.objections.specs.initial.v1.md",
        "forkcast.dark-mode.specs.initial.v1.md",
    ];
    let result = check_for_duplicates(
        &files,
        "{repo}.{feature}.{type}.{variant}.v{N}",
        "{repo}.{type}.{feature}.{variant}.v{N}",
    );
    assert!(result.is_ok());
}

#[test]
fn test_no_files_match_pattern() {
    let files = vec![
        "forkcast.objections.specs.initial.v1.md",
        "kittynode.tor.specs.initial.v1.md",
    ];
    let result = match_files(&files, "{foo}.{bar}.v{N}");
    assert!(result.is_empty());
}

#[test]
fn test_partial_match() {
    let files = vec![
        "forkcast.objections.specs.initial.v1.md",
        "README.md",
        "prompts.md",
    ];
    let result = match_files(&files, "{repo}.{feature}.{type}.{variant}.v{N}");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "forkcast.objections.specs.initial.v1.md");
}

#[test]
fn test_target_exists() {
    let existing = vec!["a.specs.b.c.v1.md"];
    let renames = vec![("a.b.specs.c.v1.md", "a.specs.b.c.v1.md")];
    let result = check_existing_targets(&renames, &existing);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

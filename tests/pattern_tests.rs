use axon::pattern::{is_valid_filename, parse_filename};

#[test]
fn test_valid_pattern() {
    assert!(is_valid_filename("forkcast.objections.specs.initial.v1.md"));
    assert!(is_valid_filename("kittynode.tor.specs.initial.v3.md"));
    assert!(is_valid_filename(
        "forkcast.dark-mode-fix.specs.shadcn.v2.md"
    ));
}

#[test]
fn test_invalid_pattern() {
    assert!(!is_valid_filename("README.md"));
    assert!(!is_valid_filename("prompts.md"));
    assert!(!is_valid_filename("bad-filename.md"));
    assert!(!is_valid_filename("missing.version.specs.initial.md"));
}

#[test]
fn test_parse_filename() {
    let parsed = parse_filename("forkcast.objections.specs.initial.v1.md").unwrap();
    assert_eq!(parsed.repo, "forkcast");
    assert_eq!(parsed.feature, "objections");
    assert_eq!(parsed.doc_type, "specs");
    assert_eq!(parsed.variant, "initial");
    assert_eq!(parsed.version, 1);
}

use axon::pattern::{is_valid_filename, parse_filename, ParsedFilename};

#[test]
fn test_valid_pattern() {
    assert!(is_valid_filename(
        "forkcast.feat.objections.specs.initial.v1.md"
    ));
    assert!(is_valid_filename(
        "kittynode.feat.tor.specs.initial.v3.md"
    ));
    assert!(is_valid_filename(
        "forkcast.feat.dark-mode-fix.specs.shadcn.v2.md"
    ));
}

#[test]
fn test_invalid_pattern() {
    assert!(!is_valid_filename("README.md"));
    assert!(!is_valid_filename("prompts.md"));
    assert!(!is_valid_filename("bad-filename.md"));
    assert!(!is_valid_filename("missing.version.specs.initial.md"));
    assert!(!is_valid_filename("forkcast.objections.specs.initial.v1.md"));
}

#[test]
fn test_parse_filename() {
    let parsed = parse_filename("forkcast.feat.objections.specs.initial.v1.md").unwrap();
    match parsed {
        ParsedFilename::Feat(f) => {
            assert_eq!(f.repo, "forkcast");
            assert_eq!(f.feature, "objections");
            assert_eq!(f.doc_type, "specs");
            assert_eq!(f.variant, "initial");
            assert_eq!(f.version, 1);
        }
        ParsedFilename::Sop(_) => panic!("expected feat filename"),
    }
}

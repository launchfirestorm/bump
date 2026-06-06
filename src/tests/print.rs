use super::*;
use crate::print::{self, PrintIsolate, PrintOptions};

fn semver_with_label_position(position: LabelPosition) -> Version {
    let mut version = make_semver("v", 1, 2, 3, 0);
    version.label.position = position;
    version
}

#[test]
fn format_print_no_prefix_with_suffix() {
    let repo = create_temp_git_repo(false);
    let mut version = make_semver("v", 1, 2, 3, 0);
    version.path = repo.path().join("bump.toml");
    let sha = git_rev_parse_short_in(repo.path());

    let opts = PrintOptions {
        omit_prefix: true,
        with_suffix: true,
        ..Default::default()
    };
    assert_eq!(
        print::format(&version, &opts).unwrap(),
        format!("1.2.3+{sha}")
    );
}

#[test]
fn format_print_no_phase_with_timestamp() {
    let version = make_semver("v", 1, 2, 3, 0);
    let opts = PrintOptions {
        omit_phase: true,
        with_timestamp: true,
        ..Default::default()
    };
    assert_eq!(
        print::format(&version, &opts).unwrap(),
        format!("v1.2.3  {}", version.timestamp.last)
    );
}

#[test]
fn format_print_full_ignores_other_flags() {
    let repo = create_temp_git_repo(false);
    let mut version = make_semver("v", 1, 2, 3, 0);
    version.path = repo.path().join("bump.toml");
    let sha = git_rev_parse_short_in(repo.path());

    let opts = PrintOptions {
        full: true,
        isolate: Some(PrintIsolate::Base),
        omit_prefix: true,
        omit_phase: true,
        ..Default::default()
    };
    assert_eq!(
        print::format(&version, &opts).unwrap(),
        format!("v1.2.3+{sha}  {}", version.timestamp.last)
    );
}

#[test]
fn format_print_full_with_label_before_phase() {
    let repo = create_temp_git_repo(false);
    let mut version = semver_with_label_position(LabelPosition::BeforePhase);
    version.path = repo.path().join("bump.toml");
    let sha = git_rev_parse_short_in(repo.path());

    let opts = PrintOptions {
        full: true,
        with_label: Some("DEV".to_string()),
        ..Default::default()
    };
    assert_eq!(
        print::format(&version, &opts).unwrap(),
        format!("v1.2.3DEV+{sha}  {}", version.timestamp.last)
    );
}

#[test]
fn format_print_only_base_with_label_is_invalid_via_parse() {
    use crate::cli;

    let cmd = cli::cli();
    let matches = cmd
        .get_matches_from(["bump", "print", "--only-base", "--with-label", "DEV"]);
    let print_matches = matches.subcommand_matches("print").unwrap();
    let err = print::parse_options(print_matches).unwrap_err();
    match err {
        BumpError::LogicError(msg) => {
            assert!(msg.contains("Cannot combine"));
        }
        _ => panic!("expected LogicError"),
    }
}

#[test]
fn format_print_with_label_at_each_position() {
    let cases = [
        (LabelPosition::BeforeBase, "vDEV1.2.3"),
        (LabelPosition::AfterBase, "v1.2.3DEV"),
        (LabelPosition::BeforePhase, "v1.2.3DEV"),
        (LabelPosition::AfterPhase, "v1.2.3DEV"),
    ];

    for (position, expected) in cases {
        let version = semver_with_label_position(position);
        let opts = PrintOptions {
            with_label: Some("DEV".to_string()),
            ..Default::default()
        };
        assert_eq!(
            print::format(&version, &opts).unwrap(),
            expected,
            "position {:?}",
            position
        );
    }
}

#[test]
fn label_position_round_trips_on_save() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 0, 0, 0));

    let mut version = Version::from_file(&bump_path).unwrap();
    version.label.position = LabelPosition::AfterBase;
    version.bump(&BumpType::Patch).unwrap();
    version.to_file().unwrap();

    let reloaded = Version::from_file(&bump_path).unwrap();
    assert_eq!(reloaded.label.position, LabelPosition::AfterBase);
    let content = std::fs::read_to_string(&bump_path).unwrap();
    assert!(!content.contains("DEV"));
    assert!(!content.contains("value"));
}

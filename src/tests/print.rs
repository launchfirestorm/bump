use super::*;
use crate::print::{self, PrintOptions};

fn semver_with_label_position(position: LabelPosition) -> Version {
    let mut version = make_semver("v", 1, 2, 3, 0);
    version.label.position = position;
    version
}

fn format_with(version: &Version, opts: PrintOptions) -> String {
    print::format(version, &opts).unwrap()
}

#[test]
fn regular_includes_prefix_base_and_phase() {
    let mut version = make_semver("v", 1, 2, 3, 2);
    version.phase.separator = "-".to_string();
    version.phase.delimiter = "-".to_string();

    assert_eq!(
        print::to_string(&version, PrintType::Regular).unwrap(),
        "v1.2.3-rc-2"
    );
}

#[test]
fn no_prefix_omits_prefix() {
    let version = make_semver("v", 1, 2, 3, 0);
    assert_eq!(print::to_string(&version, PrintType::NoPrefix).unwrap(), "1.2.3");
}

#[test]
fn with_suffix_uses_git_sha_in_repo() {
    let repo = create_temp_git_repo(false);
    let mut version = make_semver("v", 1, 2, 3, 0);
    version.path = repo.path().join("bump.toml");

    let sha = git_rev_parse_short_in(repo.path());
    assert_eq!(
        format_with(
            &version,
            PrintOptions {
                with_suffix: true,
                ..Default::default()
            },
        ),
        format!("v1.2.3+{sha}")
    );
}

#[test]
fn with_suffix_fails_outside_git_repo() {
    let _repo = create_temp_dir();
    let version = make_semver("v", 1, 2, 3, 0);

    let err = print::format(
        &version,
        &PrintOptions {
            with_suffix: true,
            ..Default::default()
        },
    )
    .unwrap_err();
    match err {
        BumpError::Git(msg) => assert!(msg.contains("Not a git repository")),
        _ => panic!("expected Git error"),
    }
}

#[test]
fn no_prefix_with_suffix() {
    let repo = create_temp_git_repo(false);
    let mut version = make_semver("v", 1, 2, 3, 0);
    version.path = repo.path().join("bump.toml");
    let sha = git_rev_parse_short_in(repo.path());

    let opts = PrintOptions {
        no_prefix: true,
        with_suffix: true,
        ..Default::default()
    };
    assert_eq!(format_with(&version, opts), format!("1.2.3+{sha}"));
}

#[test]
fn no_phase_with_timestamp() {
    let version = make_semver("v", 1, 2, 3, 0);
    let opts = PrintOptions {
        no_phase: true,
        with_timestamp: true,
        ..Default::default()
    };
    assert_eq!(
        format_with(&version, opts),
        format!("v1.2.3  {}", version.timestamp.last)
    );
}

#[test]
fn full_overrides_segment_flags() {
    let repo = create_temp_git_repo(false);
    let mut version = make_semver("v", 1, 2, 3, 0);
    version.path = repo.path().join("bump.toml");
    let sha = git_rev_parse_short_in(repo.path());

    let opts = PrintOptions {
        full: true,
        only_base: true,
        no_prefix: true,
        no_phase: true,
        ..Default::default()
    };
    assert_eq!(
        format_with(&version, opts),
        format!("v1.2.3+{sha}  {}", version.timestamp.last)
    );
}

#[test]
fn full_with_label_before_phase() {
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
        format_with(&version, opts),
        format!("v1.2.3DEV+{sha}  {}", version.timestamp.last)
    );
}

#[test]
fn only_base_with_label_returns_base_only() {
    let version = semver_with_label_position(LabelPosition::AfterBase);
    let opts = PrintOptions {
        only_base: true,
        with_label: Some("DEV".to_string()),
        ..Default::default()
    };
    assert_eq!(format_with(&version, opts), "1.2.3");
}

#[test]
fn with_label_at_each_position() {
    let cases = [
        (LabelPosition::BeforePrefix, "DEVv1.2.3"),
        (LabelPosition::AfterPrefix, "vDEV1.2.3"),
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
            format_with(&version, opts),
            expected,
            "position {:?}",
            position
        );
    }
}

#[test]
fn label_suppressed_when_anchored_segment_omitted() {
    let cases = [
        (
            LabelPosition::AfterPrefix,
            PrintOptions {
                no_prefix: true,
                with_label: Some("DEV".to_string()),
                ..Default::default()
            },
            "1.2.3",
        ),
        (
            LabelPosition::BeforeBase,
            PrintOptions {
                no_prefix: true,
                with_label: Some("DEV".to_string()),
                ..Default::default()
            },
            "DEV1.2.3",
        ),
        (
            LabelPosition::BeforePhase,
            PrintOptions {
                no_phase: true,
                with_label: Some("DEV".to_string()),
                ..Default::default()
            },
            "v1.2.3",
        ),
        (
            LabelPosition::AfterBase,
            PrintOptions {
                no_phase: true,
                with_label: Some("DEV".to_string()),
                ..Default::default()
            },
            "v1.2.3DEV",
        ),
    ];

    for (position, opts, expected) in cases {
        let version = semver_with_label_position(position);
        assert_eq!(
            format_with(&version, opts),
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
    write_semver_config(&bump_path, (1, 0, 0, 0));

    let mut version = Version::from_file(&bump_path).unwrap();
    version.label.position = LabelPosition::AfterBase;
    version.bump(&BumpType::Patch).unwrap();
    version.to_file().unwrap();

    let reloaded = Version::from_file(&bump_path).unwrap();
    assert_eq!(reloaded.label.position, LabelPosition::AfterBase);
    let content = std::fs::read_to_string(&bump_path).unwrap();
    assert!(content.contains("position = \"after-base\""));
}

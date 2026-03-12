use super::*;
use tempfile::TempDir;

#[test]
fn from_file_reads_semver_variant() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 2, 3, 0));

    let version = Version::from_file(&bump_path).unwrap();

    match version.version_type {
        VersionType::SemVer(semver) => {
            assert_eq!(semver.format.prefix, "v");
            assert_eq!(semver.version.major, 1);
            assert_eq!(semver.version.minor, 2);
            assert_eq!(semver.version.patch, 3);
            assert_eq!(semver.version.candidate, 0);
        }
        _ => panic!("expected SemVer variant"),
    }
}

#[test]
fn from_file_missing_file_returns_logic_error() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("missing.toml");

    let err = Version::from_file(&bump_path).unwrap_err();
    match err {
        BumpError::LogicError(msg) => assert!(msg.contains("Configuration file not found")),
        _ => panic!("expected LogicError"),
    }
}

#[test]
fn from_file_rejects_both_semver_and_calver_sections() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    let content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 0
patch = 0
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"

[calver.format]
prefix = ""
delimiter = "."
year = "%Y"

[calver.version]
year = "2026"

[calver.conflict]
revision = 0
delimiter = "-"
"#;
    write_bump_toml(&bump_path, content);

    let err = Version::from_file(&bump_path).unwrap_err();
    match err {
        BumpError::ParseError(msg) => assert!(msg.contains("Cannot have both [semver] and [calver]")),
        _ => panic!("expected ParseError"),
    }
}

#[test]
fn file_init_writes_semver_template() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    let version = Version {
        version_type: VersionType::SemVer(default_semver("v", 1, 2, 3, 4)),
        path: bump_path.clone(),
    };

    version.file_init().unwrap();

    let content = std::fs::read_to_string(&bump_path).unwrap();
    assert!(content.contains("[semver.format]"));
    assert!(content.contains("major = 1"));
    assert!(content.contains("candidate = 4"));
    assert!(content.contains("[semver.development]"));
}

#[test]
fn to_string_semver_in_non_git_repo_is_stable() {
    let _repo = create_temp_dir();
    let version = make_semver("v", 1, 2, 3, 0);

    let text = version.to_string().unwrap();
    assert_eq!(text, "v1.2.3");
}

#[test]
fn to_string_semver_candidate_in_non_git_repo() {
    let _repo = create_temp_dir();
    let version = make_semver("v", 1, 2, 3, 2);

    let text = version.to_string().unwrap();
    assert_eq!(text, "v1.2.3-rc2");
}

#[test]
fn to_string_semver_with_git_repo_has_development_suffix_when_untagged() {
    let repo = create_temp_git_repo(false);
    let version = Version {
        version_type: VersionType::SemVer(default_semver("v", 1, 2, 3, 0)),
        path: repo.path().join("bump.toml"),
    };

    let text = version.to_string().unwrap();
    let sha = git_rev_parse_short_in(repo.path());
    assert_eq!(text, format!("v1.2.3+{}", sha));
}

#[test]
fn to_base_string_for_semver() {
    let version = make_semver("v", 4, 5, 6, 0);
    assert_eq!(version.to_base_string().unwrap(), "4.5.6");
}

#[test]
fn get_timestamp_returns_formatted_timestamp_for_semver() {
    let semver = SemVer {
        format: crate::version::SemVerFormat {
            prefix: "v".to_string(),
            delimiter: ".".to_string(),
            timestamp: Some("%Y".to_string()),
        },
        version: crate::version::SemVerVersion {
            major: 1,
            minor: 0,
            patch: 0,
            candidate: 0,
        },
        candidate: crate::version::SemverCandidate {
            promotion: "minor".to_string(),
            delimiter: "-rc".to_string(),
        },
        development: crate::version::SemVerDevelopment {
            promotion: "git_sha".to_string(),
            delimiter: "+".to_string(),
        },
    };
    let version = Version {
        version_type: VersionType::SemVer(semver),
        path: "unused.toml".into(),
    };

    let stamp = version.get_timestamp().unwrap();
    assert_eq!(stamp.len(), 4);
    assert!(stamp.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn bump_semver_minor_resets_patch_and_candidate() {
    let mut version = make_semver("v", 1, 2, 9, 3);

    version.bump(&BumpType::Point(PointType::Minor)).unwrap();

    match version.version_type {
        VersionType::SemVer(semver) => {
            assert_eq!(semver.version.major, 1);
            assert_eq!(semver.version.minor, 3);
            assert_eq!(semver.version.patch, 0);
            assert_eq!(semver.version.candidate, 0);
        }
        _ => panic!("expected SemVer variant"),
    }
}

#[test]
fn bump_release_requires_existing_candidate() {
    let mut version = make_semver("v", 1, 2, 3, 0);

    let err = version.bump(&BumpType::Release).unwrap_err();
    match err {
        BumpError::LogicError(msg) => assert!(msg.contains("Cannot release without a candidate")),
        _ => panic!("expected LogicError"),
    }
}

#[test]
fn bump_candidate_increments_or_starts_based_on_state() {
    let mut fresh = make_semver("v", 1, 2, 3, 0);
    fresh.bump(&BumpType::Candidate).unwrap();
    match fresh.version_type {
        VersionType::SemVer(semver) => {
            assert_eq!(semver.version.minor, 3);
            assert_eq!(semver.version.patch, 0);
            assert_eq!(semver.version.candidate, 1);
        }
        _ => panic!("expected SemVer variant"),
    }

    let mut existing = make_semver("v", 1, 2, 3, 4);
    existing.bump(&BumpType::Candidate).unwrap();
    match existing.version_type {
        VersionType::SemVer(semver) => assert_eq!(semver.version.candidate, 5),
        _ => panic!("expected SemVer variant"),
    }
}

#[test]
fn bump_candidate_uses_major_promotion_strategy() {
    let mut version = make_semver("v", 1, 2, 3, 0);
    if let VersionType::SemVer(semver) = &mut version.version_type {
        semver.candidate.promotion = "major".to_string();
    }

    version.bump(&BumpType::Candidate).unwrap();

    match version.version_type {
        VersionType::SemVer(semver) => {
            assert_eq!(semver.version.major, 2);
            assert_eq!(semver.version.minor, 0);
            assert_eq!(semver.version.patch, 0);
            assert_eq!(semver.version.candidate, 1);
        }
        _ => panic!("expected SemVer variant"),
    }
}

#[test]
fn bump_candidate_uses_patch_promotion_strategy() {
    let mut version = make_semver("v", 1, 2, 3, 0);
    if let VersionType::SemVer(semver) = &mut version.version_type {
        semver.candidate.promotion = "patch".to_string();
    }

    version.bump(&BumpType::Candidate).unwrap();

    match version.version_type {
        VersionType::SemVer(semver) => {
            assert_eq!(semver.version.major, 1);
            assert_eq!(semver.version.minor, 2);
            assert_eq!(semver.version.patch, 4);
            assert_eq!(semver.version.candidate, 1);
        }
        _ => panic!("expected SemVer variant"),
    }
}

#[test]
fn bump_candidate_invalid_promotion_defaults_to_minor() {
    let mut version = make_semver("v", 1, 2, 7, 0);
    if let VersionType::SemVer(semver) = &mut version.version_type {
        semver.candidate.promotion = "bogus".to_string();
    }

    version.bump(&BumpType::Candidate).unwrap();

    match version.version_type {
        VersionType::SemVer(semver) => {
            assert_eq!(semver.version.major, 1);
            assert_eq!(semver.version.minor, 3);
            assert_eq!(semver.version.patch, 0);
            assert_eq!(semver.version.candidate, 1);
        }
        _ => panic!("expected SemVer variant"),
    }
}

#[test]
fn build_tag_name_semver_stable_and_candidate() {
    let stable = make_semver("v", 2, 3, 4, 0);
    assert_eq!(build_tag_name(&stable).unwrap(), "v2.3.4");

    let candidate = make_semver("v", 2, 3, 4, 5);
    assert_eq!(build_tag_name(&candidate).unwrap(), "v2.3.4-rc5");
}

#[test]
fn create_git_tag_fails_outside_git_repository() {
    let _repo = create_temp_dir();
    let version = make_semver("v", 9, 9, 9, 0);

    let err = create_git_tag(&version, None).unwrap_err();
    match err {
        BumpError::LogicError(msg) => assert!(msg.contains("Not in a git repository")),
        _ => panic!("expected LogicError"),
    }
}

#[test]
fn create_git_tag_creates_tag_and_rejects_duplicate() {
    let repo = create_temp_git_repo(false);
    let version = Version {
        version_type: VersionType::SemVer(default_semver("v", 1, 4, 2, 0)),
        path: repo.path().join("bump.toml"),
    };

    with_cwd(repo.path(), || {
        create_git_tag(&version, Some("test tag")).unwrap();
    });

    let created = run_git_in_output(repo.path(), &["tag", "--list", "v1.4.2"]);
    assert_eq!(created, "v1.4.2");

    let duplicate_err = with_cwd(repo.path(), || create_git_tag(&version, None)).unwrap_err();
    match duplicate_err {
        BumpError::Git(msg) => assert!(msg.contains("already exists")),
        _ => panic!("expected Git error"),
    }
}

use super::*;
use tempfile::TempDir;

#[test]
fn from_file_reads_calver_variant() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_calver_config(&bump_path, "2026", Some("03"), Some("12"), 0);

    let version = Version::from_file(&bump_path).unwrap();

    match version.version_type {
        VersionType::CalVer(calver) => {
            assert_eq!(calver.format.delimiter, ".");
            assert_eq!(calver.version.year, "2026");
            assert_eq!(calver.version.month.as_deref(), Some("03"));
            assert_eq!(calver.version.day.as_deref(), Some("12"));
            assert_eq!(calver.conflict.revision, 0);
        }
        _ => panic!("expected CalVer variant"),
    }
}

#[test]
fn file_init_writes_calver_template() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    let version = Version {
        version_type: VersionType::CalVer(default_calver("")),
        path: bump_path.clone(),
    };

    version.file_init().unwrap();

    let content = std::fs::read_to_string(&bump_path).unwrap();
    assert!(content.contains("[calver.format]"));
    assert!(content.contains("[calver.version]"));
    assert!(content.contains("[calver.conflict]"));
    assert!(content.contains("revision = 0"));
}

#[test]
fn to_string_calver_includes_revision_only_when_nonzero() {
    let _repo = create_temp_dir();
    let mut version = make_calver("v");

    let base = version.to_string().unwrap();
    assert!(!base.contains("-0"));

    match &mut version.version_type {
        VersionType::CalVer(calver) => calver.conflict.revision = 2,
        _ => panic!("expected CalVer variant"),
    }

    let with_revision = version.to_string().unwrap();
    assert!(with_revision.ends_with("-2"));
}

#[test]
fn bump_calendar_same_day_increments_revision() {
    let now = chrono::Utc::now();
    let year = now.format("%Y").to_string();
    let month = now.format("%m").to_string();
    let day = now.format("%d").to_string();

    let mut version = Version {
        version_type: VersionType::CalVer(CalVer {
            format: crate::version::CalVerFormat {
                prefix: "".to_string(),
                delimiter: ".".to_string(),
                year: "%Y".to_string(),
                month: Some("%m".to_string()),
                day: Some("%d".to_string()),
            },
            version: crate::version::CalVerVersion {
                year,
                month: Some(month),
                day: Some(day),
            },
            conflict: crate::version::CalVerConflict {
                revision: 7,
                delimiter: "-".to_string(),
            },
        }),
        path: "unused.toml".into(),
    };

    version.bump(&BumpType::Calendar).unwrap();

    match version.version_type {
        VersionType::CalVer(calver) => assert_eq!(calver.conflict.revision, 8),
        _ => panic!("expected CalVer variant"),
    }
}

#[test]
fn bump_calendar_new_day_resets_revision_and_updates_date() {
    let now = chrono::Utc::now();
    let previous = now - chrono::Duration::days(1);

    let mut version = Version {
        version_type: VersionType::CalVer(CalVer {
            format: crate::version::CalVerFormat {
                prefix: "".to_string(),
                delimiter: ".".to_string(),
                year: "%Y".to_string(),
                month: Some("%m".to_string()),
                day: Some("%d".to_string()),
            },
            version: crate::version::CalVerVersion {
                year: previous.format("%Y").to_string(),
                month: Some(previous.format("%m").to_string()),
                day: Some(previous.format("%d").to_string()),
            },
            conflict: crate::version::CalVerConflict {
                revision: 5,
                delimiter: "-".to_string(),
            },
        }),
        path: "unused.toml".into(),
    };

    version.bump(&BumpType::Calendar).unwrap();

    match version.version_type {
        VersionType::CalVer(calver) => {
            assert_eq!(calver.conflict.revision, 0);
            assert_eq!(calver.version.year, now.format("%Y").to_string());
            assert_eq!(calver.version.month, Some(now.format("%m").to_string()));
            assert_eq!(calver.version.day, Some(now.format("%d").to_string()));
        }
        _ => panic!("expected CalVer variant"),
    }
}

#[test]
fn bump_calver_rejects_non_calendar_bumps() {
    let mut version = make_calver("");

    for bump in [
        BumpType::Point(PointType::Major),
        BumpType::Candidate,
        BumpType::Release,
        BumpType::Prefix("x".to_string()),
    ] {
        let err = version.bump(&bump).unwrap_err();
        match err {
            BumpError::LogicError(msg) => {
                assert!(msg.contains("CalVer only supports --calendar"));
            }
            _ => panic!("expected LogicError"),
        }
    }
}

#[test]
fn to_base_string_calver_returns_error() {
    let version = make_calver("");
    let err = version.to_base_string().unwrap_err();

    match err {
        BumpError::LogicError(msg) => {
            assert!(msg.contains("base version only applies to semantic versioning"));
        }
        _ => panic!("expected LogicError"),
    }
}

#[test]
fn get_timestamp_on_calver_returns_error() {
    let version = make_calver("");
    let err = version.get_timestamp().unwrap_err();

    match err {
        BumpError::LogicError(msg) => {
            assert!(msg.contains("calendar versioning"));
        }
        _ => panic!("expected LogicError"),
    }
}

#[test]
fn build_tag_name_for_calver_uses_rendered_version_string() {
    let mut version = make_calver("v");

    let stable_tag = build_tag_name(&version).unwrap();
    let stable_text = version.to_string().unwrap();
    assert_eq!(stable_tag, stable_text);

    if let VersionType::CalVer(calver) = &mut version.version_type {
        calver.conflict.revision = 3;
    }
    let conflict_tag = build_tag_name(&version).unwrap();
    assert!(conflict_tag.ends_with("-3"));
}

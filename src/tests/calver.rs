use super::*;
use chrono::Datelike;

#[test]
fn calendar_bump_updates_date_fields_for_calver() {
    let mut version = make_calver("");
    let before = chrono::Utc::now();

    version.bump(&BumpType::Calendar).unwrap();

    assert_eq!(version.version.mode, VersionMode::Calver);
    assert_eq!(version.version.major, before.year().cast_unsigned());
    assert_eq!(version.version.minor, Some(before.month()));
    assert_eq!(version.version.patch, Some(before.day()));
}

#[test]
fn calendar_bump_rejected_for_semver() {
    let mut version = make_semver("v", 1, 2, 3, 0);

    let err = version.bump(&BumpType::Calendar).unwrap_err();
    match err {
        BumpError::LogicError(msg) => {
            assert!(msg.contains("Operation only valid for version.type = 'calver'"));
        }
        _ => panic!("expected LogicError"),
    }
}

#[test]
fn calendar_bump_increments_phase_distance_when_same_day() {
    let now = chrono::Utc::now();
    let mut version = Version {
        path: "test.toml".into(),
        timestamp: TimestampTable {
            format: "%Y-%m-%d %H:%M:%S %Z".to_string(),
            last: "2026-01-01 00:00:00 UTC".to_string(),
        },
        version: VersionTable {
            mode: VersionMode::Calver,
            prefix: String::new(),
            delimiter: ".".to_string(),
            major: now.year().cast_unsigned(),
            minor: Some(now.month()),
            patch: Some(now.day()),
        },
        phase: PhaseTable {
            prefix: "-".to_string(),
            name: String::new(),
            delimiter: "-".to_string(),
            distance: 4,
        },
        suffix: SuffixTable {
            mode: SuffixMode::GitSha,
            delimiter: "+".to_string(),
        },
    };

    version.bump(&BumpType::Calendar).unwrap();

    assert_eq!(version.phase.distance, 5);
}

#[test]
fn to_file_calver_remaps_major_minor_patch_keys() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    let content = r#"[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"
last = "2026-01-01 00:00:00 UTC"

[version]
mode = "calver"
prefix = ""
delimiter = "."
major = 2026
minor = 6
patch = 5

[phase]
prefix = "-"
name = ""
delimiter = "-"
distance = 0

[suffix]
mode = "git_sha"
delimiter = "+"
"#;
    write_bump_toml(&bump_path, content);

    let version = Version::from_file(&bump_path).unwrap();
    version.to_file().unwrap();

    let rewritten = std::fs::read_to_string(&bump_path).unwrap();
    let parsed: toml::Value = toml::from_str(&rewritten).unwrap();
    let table = parsed.get("version").unwrap().as_table().unwrap();

    assert_eq!(
        table.get("year").and_then(toml::Value::as_integer),
        Some(2026)
    );
    assert_eq!(
        table.get("month").and_then(toml::Value::as_integer),
        Some(6)
    );
    assert_eq!(table.get("day").and_then(toml::Value::as_integer), Some(5));
    assert!(!table.contains_key("major"));
    assert!(!table.contains_key("minor"));
    assert!(!table.contains_key("patch"));
}

use super::*;
use chrono::Datelike;

#[test]
fn calendar_bump_updates_date_fields_for_calver() {
    let mut version = make_calver("");
    let before = chrono::Utc::now();

    version.bump(&BumpType::Calendar).unwrap();

    assert_eq!(version.version._type, "calver");
    assert_eq!(version.version.major, before.year() as u32);
    assert_eq!(version.version.minor, Some(before.month()));
    assert_eq!(version.version.patch, Some(before.day()));
}

#[test]
fn calendar_bump_rejected_for_semver() {
    let mut version = make_semver("v", 1, 2, 3, 0);

    let err = version.bump(&BumpType::Calendar).unwrap_err();
    match err {
        BumpError::LogicError(msg) => {
            assert!(msg.contains("Calendar bump is only applicable"));
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
            _type: "calver".to_string(),
            prefix: "".to_string(),
            delimiter: ".".to_string(),
            major: now.year() as u32,
            minor: Some(now.month()),
            patch: Some(now.day()),
        },
        phase: PhaseTable {
            name: "".to_string(),
            delimiter: "-".to_string(),
            distance: 4,
        },
        suffix: SuffixTable {
            _type: "git_sha".to_string(),
            delimiter: "+".to_string(),
        },
    };

    version.bump(&BumpType::Calendar).unwrap();

    assert_eq!(version.phase.distance, 5);
}

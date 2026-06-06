use super::*;
use tempfile::TempDir;

#[test]
fn from_file_reads_semver_schema() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 2, 3, 4));

    let version = Version::from_file(&bump_path).unwrap();

    assert_eq!(version.version.mode, "semver");
    assert_eq!(version.version.prefix, "v");
    assert_eq!(version.version.major, 1);
    assert_eq!(version.version.minor, Some(2));
    assert_eq!(version.version.patch, Some(3));
    assert_eq!(version.phase.name, "rc");
    assert_eq!(version.phase.distance, 4);
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
fn from_file_rejects_invalid_version_type() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    let content = r#"[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"
last = "2026-01-01 00:00:00 UTC"

[version]
mode = "nope"
prefix = "v"
delimiter = "."
major = 1
minor = 0
patch = 0

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

    let err = Version::from_file(&bump_path).unwrap_err();
    match err {
        BumpError::ParseError(msg) => assert!(msg.contains("Invalid version type")),
        _ => panic!("expected ParseError"),
    }
}

#[test]
fn from_file_rejects_invalid_suffix_type() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    let content = r#"[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"
last = "2026-01-01 00:00:00 UTC"

[version]
mode = "semver"
prefix = "v"
delimiter = "."
major = 1
minor = 0
patch = 0

[phase]
prefix = "-"
name = ""
delimiter = "-"
distance = 0

[suffix]
mode = "unknown"
delimiter = "+"
"#;
    write_bump_toml(&bump_path, content);

    let err = Version::from_file(&bump_path).unwrap_err();
    match err {
        BumpError::ParseError(msg) => assert!(msg.contains("Invalid suffix type")),
        _ => panic!("expected ParseError"),
    }
}

#[test]
fn create_file_writes_template() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    let version = Version::default(&bump_path);

    version.create_file().unwrap();

    let content = std::fs::read_to_string(&bump_path).unwrap();
    assert!(content.contains("[timestamp]"));
    assert!(content.contains("[version]"));
    assert!(content.contains("[phase]"));
    assert!(content.contains("[suffix]"));
}

#[test]
fn to_file_semver_remaps_year_month_day_keys() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    let content = r#"[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"
last = "2026-01-01 00:00:00 UTC"

[version]
mode = "semver"
prefix = "v"
delimiter = "."
year = 2026
month = 6
day = 5

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
        table.get("major").and_then(toml::Value::as_integer),
        Some(2026)
    );
    assert_eq!(
        table.get("minor").and_then(toml::Value::as_integer),
        Some(6)
    );
    assert_eq!(
        table.get("patch").and_then(toml::Value::as_integer),
        Some(5)
    );
    assert!(!table.contains_key("year"));
    assert!(!table.contains_key("month"));
    assert!(!table.contains_key("day"));
}

#[test]
fn to_string_regular_uses_prefix_base_and_phase() {
    let mut version = make_semver("v", 1, 2, 3, 2);
    version.phase.prefix = "-".to_string();
    version.phase.delimiter = "-".to_string();

    assert_eq!(
        version.to_string(&PrintType::Regular).unwrap(),
        "v1.2.3-rc-2"
    );
}

#[test]
fn to_string_no_prefix_removes_prefix() {
    let version = make_semver("v", 1, 2, 3, 0);
    assert_eq!(version.to_string(&PrintType::NoPrefix).unwrap(), "1.2.3");
}

#[test]
fn to_string_with_suffix_uses_git_sha_in_git_repo() {
    let repo = create_temp_git_repo(false);
    let mut version = make_semver("v", 1, 2, 3, 0);
    version.path = repo.path().join("bump.toml");

    let sha = git_rev_parse_short_in(repo.path());
    assert_eq!(
        version.to_string(&PrintType::WithSuffix).unwrap(),
        format!("v1.2.3+{sha}")
    );
}

#[test]
fn to_string_with_suffix_fails_outside_git_repo() {
    let _repo = create_temp_dir();
    let version = make_semver("v", 1, 2, 3, 0);

    let err = version.to_string(&PrintType::WithSuffix).unwrap_err();
    match err {
        BumpError::Git(msg) => assert!(msg.contains("Not a git repository")),
        _ => panic!("expected Git error"),
    }
}

#[test]
fn bump_minor_resets_patch_and_phase() {
    let mut version = make_semver("v", 1, 2, 9, 7);

    version.bump(&BumpType::Minor).unwrap();

    assert_eq!(version.version.major, 1);
    assert_eq!(version.version.minor, Some(3));
    assert_eq!(version.version.patch, Some(0));
    assert_eq!(version.phase.name, "");
    assert_eq!(version.phase.distance, 0);
}

#[test]
fn bump_patch_increments_patch_and_clears_phase() {
    let mut version = make_semver("v", 1, 2, 3, 2);

    version.bump(&BumpType::Patch).unwrap();

    assert_eq!(version.version.patch, Some(4));
    assert_eq!(version.phase.name, "");
    assert_eq!(version.phase.distance, 0);
}

#[test]
fn bump_phase_increment_mode_increments_distance() {
    let mut version = make_semver("v", 1, 2, 3, 1);

    version
        .bump(&BumpType::Phase("__increment__".to_string()))
        .unwrap();

    assert_eq!(version.phase.name, "rc");
    assert_eq!(version.phase.distance, 2);
}

#[test]
fn bump_phase_new_name_switches_and_resets_distance() {
    let mut version = make_semver("v", 1, 2, 3, 5);

    version.bump(&BumpType::Phase("beta".to_string())).unwrap();

    assert_eq!(version.phase.name, "beta");
    assert_eq!(version.phase.distance, 1);
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
    let mut version = make_semver("v", 1, 4, 2, 0);
    version.path = repo.path().join("bump.toml");

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

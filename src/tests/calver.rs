// CalVer-specific tests
use super::*;
use tempfile::TempDir;

#[test]
fn test_init_calver_creates_proper_structure() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create a new CalVer version by initializing
    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::CalVer { revision: 0 },
        path: config_path.clone(),
        config: crate::version::default_calver_config("v".to_string()),
    };
    
    version.file_init().unwrap();

    // Verify file was created
    assert!(config_path.exists());

    // Read the file and verify structure
    let content = fs::read_to_string(&config_path).unwrap();

    // Check for header
    assert!(content.contains("https://github.com/launchfirestorm/bump"));

    // Check for [calver.format] section
    assert!(content.contains("[calver.format]"));
    assert!(content.contains("prefix = \"\""));
    assert!(content.contains("year = \"%Y\""));
    assert!(content.contains("month = \"%m\""));
    assert!(content.contains("day = \"%d\""));
    
    // Check for [calver.version] section
    assert!(content.contains("[calver.version]"));
    assert!(content.contains("NOTE: This section is modified by the bump command"));

    // Check for conflict section with comments
    assert!(content.contains("[calver.conflict]"));
    assert!(content.contains("revision = 0"));
    assert!(content.contains("delimiter = \"-\""));

    // Verify it can be read back
    let read_version = Version::from_file(&config_path).unwrap();
    match &read_version.version_type {
        VersionType::CalVer { revision } => {
            assert_eq!(*revision, 0);
        },
        _ => panic!("Expected CalVer version type"),
    }
    assert_eq!(read_version.prefix, "");
}

#[test]
fn test_calendar_bump_updates_to_current_date() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file with old date
    let content = r#"[calver.format]
prefix = ""
delimiter = "."
year = "%Y"
month = "%m"
day = "%d"

[calver.version]
year = "2024"
month = "01"
day = "15"

[calver.conflict]
revision = 0
delimiter = "-"
"#;
    fs::write(&config_path, content).unwrap();

    // Load and bump with calendar
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    // Verify date was updated to current date
    let updated_version = Version::from_file(&config_path).unwrap();
    if let Config::CalVer(config) = &updated_version.config {
        let now = chrono::Utc::now();
        assert_eq!(config.version.year, now.format("%Y").to_string());
        assert_eq!(config.version.month, Some(now.format("%m").to_string()));
        assert_eq!(config.version.day, Some(now.format("%d").to_string()));
        
        // Revision should be reset to 0 for new date
        if let VersionType::CalVer { revision } = updated_version.version_type {
            assert_eq!(revision, 0);
        }
    } else {
        panic!("Expected CalVer config");
    }
}

#[test]
fn test_same_day_bump_increments_revision() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file with current date
    let now = chrono::Utc::now();
    let content = format!(r#"[calver.format]
prefix = ""
delimiter = "."
year = "%Y"
month = "%m"
day = "%d"

[calver.version]
year = "{}"
month = "{}"
day = "{}"

[calver.conflict]
revision = 0
delimiter = "-"
"#, now.format("%Y"), now.format("%m"), now.format("%d"));
    fs::write(&config_path, content).unwrap();

    // First bump - should increment revision since date matches
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    let updated = Version::from_file(&config_path).unwrap();
    if let VersionType::CalVer { revision } = updated.version_type {
        assert_eq!(revision, 1);
    } else {
        panic!("Expected CalVer version type");
    }

    // Second bump - should increment to 2
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    let updated = Version::from_file(&config_path).unwrap();
    if let VersionType::CalVer { revision } = updated.version_type {
        assert_eq!(revision, 2);
    } else {
        panic!("Expected CalVer version type");
    }
}

#[test]
fn test_different_day_resets_revision() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file with yesterday's date and revision
    let now = chrono::Utc::now();
    let yesterday = now - chrono::Duration::days(1);
    let content = format!(r#"[calver.format]
prefix = ""
delimiter = "."
year = "%Y"
month = "%m"
day = "%d"

[calver.version]
year = "{}"
month = "{}"
day = "{}"

[calver.conflict]
revision = 5
delimiter = "-"
"#, yesterday.format("%Y"), yesterday.format("%m"), yesterday.format("%d"));
    fs::write(&config_path, content).unwrap();

    // Bump - should reset revision to 0 for new date
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    let updated = Version::from_file(&config_path).unwrap();
    if let Config::CalVer(config) = &updated.config {
        assert_eq!(config.version.year, now.format("%Y").to_string());
        assert_eq!(config.version.month, Some(now.format("%m").to_string()));
        assert_eq!(config.version.day, Some(now.format("%d").to_string()));
        
        if let VersionType::CalVer { revision } = updated.version_type {
            assert_eq!(revision, 0, "Revision should reset to 0 for new date");
        }
    } else {
        panic!("Expected CalVer config");
    }
}

#[test]
fn test_year_only_format() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file with year-only format
    let content = r#"[calver.format]
prefix = "v"
delimiter = "."
year = "%Y"

[calver.version]
year = "2024"

[calver.conflict]
revision = 0
delimiter = "-"
"#;
    fs::write(&config_path, content).unwrap();

    // Bump to current year
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    // Check version string
    let updated = Version::from_file(&config_path).unwrap();
    let version_str = updated.to_string(&BumpType::Calendar);
    let now = chrono::Utc::now();
    assert_eq!(version_str, format!("v{}", now.format("%Y")));

    // Verify stored version
    if let Config::CalVer(config) = &updated.config {
        assert_eq!(config.version.year, now.format("%Y").to_string());
        assert_eq!(config.version.month, None);
        assert_eq!(config.version.day, None);
    } else {
        panic!("Expected CalVer config");
    }
}

#[test]
fn test_year_month_format() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file with year.month format
    let content = r#"[calver.format]
prefix = ""
delimiter = "."
year = "%Y"
month = "%m"

[calver.version]
year = "2024"
month = "01"

[calver.conflict]
revision = 0
delimiter = "-"
"#;
    fs::write(&config_path, content).unwrap();

    // Bump to current year.month
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    // Check version string
    let updated = Version::from_file(&config_path).unwrap();
    let version_str = updated.to_string(&BumpType::Calendar);
    let now = chrono::Utc::now();
    assert_eq!(version_str, now.format("%Y.%m").to_string());

    // Verify stored version
    if let Config::CalVer(config) = &updated.config {
        assert_eq!(config.version.year, now.format("%Y").to_string());
        assert_eq!(config.version.month, Some(now.format("%m").to_string()));
        assert_eq!(config.version.day, None);
    } else {
        panic!("Expected CalVer config");
    }
}

#[test]
fn test_calendar_with_revision_in_version_string() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file
    let now = chrono::Utc::now();
    let content = format!(r#"[calver.format]
prefix = "v"
delimiter = "."
year = "%Y"
month = "%m"
day = "%d"

[calver.version]
year = "{}"
month = "{}"
day = "{}"

[calver.conflict]
revision = 0
delimiter = "-"
"#, now.format("%Y"), now.format("%m"), now.format("%d"));
    fs::write(&config_path, content).unwrap();

    // First bump - revision 1
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    let updated = Version::from_file(&config_path).unwrap();
    let version_str = updated.to_string(&BumpType::Calendar);
    assert_eq!(version_str, format!("v{}-1", now.format("%Y.%m.%d")));
}

#[test]
fn test_calver_rejects_semver_bump_types() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file
    let content = r#"[calver.format]
prefix = ""
delimiter = "."
year = "%Y"

[calver.version]
year = "2024"

[calver.conflict]
revision = 0
delimiter = "-"
"#;
    fs::write(&config_path, content).unwrap();

    let mut version = Version::from_file(&config_path).unwrap();

    // Test that Point bumps fail
    let result = version.bump(&BumpType::Point(PointType::Major));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("CalVer only supports --calendar"));

    // Reset
    let mut version = Version::from_file(&config_path).unwrap();
    let result = version.bump(&BumpType::Candidate);
    assert!(result.is_err());

    // Reset
    let mut version = Version::from_file(&config_path).unwrap();
    let result = version.bump(&BumpType::Release);
    assert!(result.is_err());
}

#[test]
fn test_calver_preserves_comments_on_bump() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file with comments
    let content = r#"# This is a custom comment
[calver.format]
prefix = ""
delimiter = "."
year = "%Y"

# Another comment
[calver.version]
year = "2024"

[calver.conflict]
revision = 0
delimiter = "-"
"#;
    fs::write(&config_path, content).unwrap();

    // Bump the version
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    // Read file and verify block comments are preserved
    let updated_content = fs::read_to_string(&config_path).unwrap();
    assert!(updated_content.contains("# This is a custom comment"));
    assert!(updated_content.contains("# Another comment"));
}

#[test]
fn test_calver_custom_delimiter() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create CalVer file with custom delimiter
    let content = r#"[calver.format]
prefix = "release_"
delimiter = "-"
year = "%Y"
month = "%m"
day = "%d"

[calver.version]
year = "2024"
month = "01"
day = "01"

[calver.conflict]
revision = 0
delimiter = "."
"#;
    fs::write(&config_path, content).unwrap();

    // Bump to current date
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    // Check version string uses custom delimiter
    let updated = Version::from_file(&config_path).unwrap();
    let version_str = updated.to_string(&BumpType::Calendar);
    let now = chrono::Utc::now();
    assert_eq!(version_str, format!("release_{}", now.format("%Y-%m-%d")));
}

#[test]
fn test_format_section_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    let now = chrono::Utc::now();
    let content = format!(r#"[calver.format]
prefix = "v"
delimiter = "."
year = "%Y"
month = "%m"
day = "%d"

[calver.version]
year = "{}"
month = "{}"
day = "{}"

[calver.conflict]
revision = 0
delimiter = "-"
"#, now.format("%Y"), now.format("%m"), now.format("%d"));
    fs::write(&config_path, content).unwrap();

    // Bump the version
    let mut version = Version::from_file(&config_path).unwrap();
    version.bump(&BumpType::Calendar).unwrap();
    version.to_file().unwrap();

    // Read the file and verify format section is preserved
    let updated_content = fs::read_to_string(&config_path).unwrap();
    assert!(updated_content.contains("prefix = \"v\""));
    assert!(updated_content.contains("delimiter = \".\""));
    
    // Verify config values
    let updated = Version::from_file(&config_path).unwrap();
    if let Config::CalVer(config) = &updated.config {
        assert_eq!(config.format.prefix, "v");
        assert_eq!(config.format.delimiter, ".");
    } else {
        panic!("Expected CalVer config");
    }
}

#[test]
fn test_format_section_inline_comments_preserved() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    let now = chrono::Utc::now();
    let content = format!(r#"[calver.format]
prefix = "v"
delimiter = "."
year = "%Y"        # strftime 4 digit year
month = "%m"       # [optional] strftime zero padded month
day = "%d"         # [optional] strftime zero padded day

[calver.version]
year = "{}"
month = "{}"
day = "{}"

[calver.conflict]
revision = 0       # increments on same day bumps
delimiter = "-"
"#, now.format("%Y"), now.format("%m"), now.format("%d"));
    fs::write(&config_path, content).unwrap();

    // Bump the version multiple times
    for _ in 0..3 {
        let mut version = Version::from_file(&config_path).unwrap();
        version.bump(&BumpType::Calendar).unwrap();
        version.to_file().unwrap();
    }

    // Read the file and verify inline comments are preserved
    let updated_content = fs::read_to_string(&config_path).unwrap();
    assert!(updated_content.contains("# strftime 4 digit year"));
    assert!(updated_content.contains("# [optional] strftime zero padded month"));
    assert!(updated_content.contains("# [optional] strftime zero padded day"));
}

#[test]
fn test_build_tag_name_calver_uses_full_version_string() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    let content = r#"[calver.format]
prefix = ""
delimiter = "."
year = "%Y"
month = "%m"
day = "%d"

[calver.version]
year = "2026"
month = "02"
day = "28"

[calver.conflict]
revision = 1
delimiter = "-"
"#;
    fs::write(&config_path, content).unwrap();

    let version = Version::from_file(&config_path).unwrap();
    let tag_name = crate::bump::build_tag_name(&version).unwrap();

    assert_eq!(tag_name, "2026.02.28-1");
}

#[test]
fn test_build_tag_name_calver_respects_custom_format_and_delimiters() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    let content = r#"[calver.format]
prefix = "release_"
delimiter = "-"
year = "%Y"
month = "%m"

[calver.version]
year = "2026"
month = "02"

[calver.conflict]
revision = 3
delimiter = "."
"#;
    fs::write(&config_path, content).unwrap();

    let version = Version::from_file(&config_path).unwrap();
    let tag_name = crate::bump::build_tag_name(&version).unwrap();

    assert_eq!(tag_name, "release_2026-02.3");
}

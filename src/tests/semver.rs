// SemVer-specific tests
use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn version_default() {
    let path = PathBuf::from("test.toml");
    let version = Version::default(&path);

    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 0);
            assert_eq!(*minor, 1);
            assert_eq!(*patch, 0);
            assert_eq!(*candidate, 0);
        }
        _ => panic!("Expected SemVer version type"),
    }
    assert_eq!(version.path, path);
    assert_eq!(version.prefix, "v");
    match &version.config {
        BumpConfig::SemVer(cfg) => {
            assert_eq!(cfg.candidate.promotion, "minor");
            assert_eq!(cfg.candidate.delimiter, "-rc");
            assert_eq!(cfg.development.promotion, "git_sha");
            assert_eq!(cfg.development.delimiter, "+");
        }
        _ => panic!("Expected SemVer config"),
    }
}

#[test]
fn version_from_file_valid() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let content = r#"[semver.format]
prefix = "prefix_"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.prefix, "prefix_");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 2);
            assert_eq!(*patch, 3);
            assert_eq!(*candidate, 0);
        }
        _ => panic!("Expected SemVer version type"),
    }
    assert_eq!(version.path, file_path);
}

#[test]
fn version_from_file_invalid_major() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = "invalid"
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::TomlError(_) => (), // Expected TOML parsing error
        _ => panic!("Expected TomlError"),
    }
}

#[test]
fn version_from_file_invalid_minor() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = "invalid"
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::TomlError(_) => (), // Expected TOML parsing error
        _ => panic!("Expected TomlError"),
    }
}

#[test]
fn version_from_file_invalid_patch() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = "invalid"
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::TomlError(_) => (), // Expected TOML parsing error
        _ => panic!("Expected TomlError"),
    }
}

#[test]
fn version_from_file_invalid_candidate() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = "invalid"

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::TomlError(_) => (), // Expected TOML parsing error
        _ => panic!("Expected TomlError"),
    }
}

#[test]
fn version_from_file_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nonexistent.toml");

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::LogicError(msg) => {
            assert!(msg.contains("Configuration file not found"));
            assert!(msg.contains("bump init"));
        }
        _ => panic!("Expected LogicError"),
    }
}

#[test]
fn version_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let config = make_default_config(1, 2, 3, 4);

    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: file_path.clone(),
        config,
    };

    version.file_init().unwrap();
    version.to_file().unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("major = 1"));
    assert!(content.contains("minor = 2"));
    assert!(content.contains("patch = 3"));
    assert!(content.contains("candidate = 4"));
    assert!(content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn version_to_string_point() {
    let config = make_default_config(1, 2, 3, 0);

    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 0,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    let version_string = version.to_string(&BumpType::Point(PointType::Patch));
    assert_eq!(version_string, "v1.2.3");
}

#[test]
fn version_to_string_candidate() {
    let config = make_default_config(1, 2, 3, 4);

    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    let version_string = version.to_string(&BumpType::Candidate);
    assert_eq!(version_string, "v1.2.3-rc4");
}

#[test]
fn version_to_header() {
    let _repo = create_temp_dir();
    let header_path = _repo.path().join("version.h");

    let config = make_default_config(1, 2, 3, 4);

    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    crate::lang::output_file(
        &crate::lang::Language::C,
        &version,
        &header_path,
    )
    .unwrap();

    let header_content = fs::read_to_string(&header_path).unwrap();
    assert!(header_content.contains("#define VERSION_MAJOR 1"));
    assert!(header_content.contains("#define VERSION_MINOR 2"));
    assert!(header_content.contains("#define VERSION_PATCH 3"));
    assert!(header_content.contains("#define VERSION_CANDIDATE 4"));
    assert!(header_content.contains("#define VERSION_STRING "));
    assert!(header_content.contains("1.2.3-rc4"));
    assert!(header_content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn resolve_path_absolute() {
    let absolute_path = if cfg!(windows) {
        "C:\\test\\path"
    } else {
        "/test/path"
    };

    let resolved = resolve_path(absolute_path);
    assert_eq!(resolved, PathBuf::from(absolute_path));
}

#[test]
fn resolve_path_relative() {
    let relative_path = "test.bumpfile";
    let resolved = resolve_path(relative_path);

    // Should be resolved relative to current directory
    assert!(resolved.is_absolute());
    assert!(resolved.to_string_lossy().ends_with("test.bumpfile"));
}

#[test]
fn test_ensure_directory_exists() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("nested").join("deep").join("file.txt");

    ensure_directory_exists(&nested_path).unwrap();

    assert!(nested_path.parent().unwrap().exists());
}

#[test]
fn bump_error_display() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let bump_error = BumpError::IoError(io_error);

    let display = format!("{bump_error}");
    assert!(display.contains("bump error: I/O >> file not found"));

    let parse_error = BumpError::ParseError("invalid MAJOR value".to_string());
    let display = format!("{parse_error}");
    assert!(display.contains("bump error: parse >> invalid MAJOR value"));

    let logic_error = BumpError::LogicError("Test error".to_string());
    let display = format!("{logic_error}");
    assert!(display.contains("bump error >> Test error"));
}

#[test]
fn bump_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let bump_error: BumpError = io_error.into();

    match bump_error {
        BumpError::IoError(_) => (), // Expected
        _ => panic!("Expected IoError"),
    }
}

#[test]
fn version_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let config = make_default_config(5, 10, 15, 2);

    let original_version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 5,
            minor: 10,
            patch: 15,
            candidate: 2,
        },
        path: file_path.clone(),
        config,
    };

    // Write to file
    original_version.file_init().unwrap();
    original_version.to_file().unwrap();

    // Read from file
    let read_version = Version::from_file(&file_path).unwrap();

    match (&original_version.version_type, &read_version.version_type) {
        (VersionType::SemVer { major: orig_major, minor: orig_minor, patch: orig_patch, candidate: orig_candidate },
         VersionType::SemVer { major: read_major, minor: read_minor, patch: read_patch, candidate: read_candidate }) => {
            assert_eq!(orig_major, read_major);
            assert_eq!(orig_minor, read_minor);
            assert_eq!(orig_patch, read_patch);
            assert_eq!(orig_candidate, read_candidate);
        },
        _ => panic!("Expected SemVer version types"),
    }

    assert_eq!(original_version.path, read_version.path);
}

#[test]
fn version_file_with_comments() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let content = r#"# This is a comment
[semver.format]
prefix = ""
delimiter = "."

[semver.version]
major = 1
# Another comment
minor = 2
patch = 3
candidate = 0
# End comment

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 2);
            assert_eq!(*patch, 3);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn version_file_with_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let content = r#"[semver.format]
prefix = ""
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.prefix, "");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 2);
            assert_eq!(*patch, 3);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn commit_sha() {
    let _repo = create_temp_git_repo(false);
    let commit_sha = get_git_commit_sha().unwrap();
    assert!(!commit_sha.is_empty(), "Commit SHA should not be empty");
    assert_eq!(commit_sha.len(), 7, "Commit SHA should be 7 characters long");
    assert!(
        commit_sha.chars().all(|c| c.is_ascii_hexdigit()),
        "Commit SHA should only contain hex digits"
    );
}

#[test]
fn test_timestamp_config_none() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    write_test_config(&file_path, (1, 0, 0, 0));

    let version = Version::from_file(&file_path).unwrap();

    // When timestamp config is not set, it should be None
    match &version.config {
        Config::SemVer(cfg) => assert!(cfg.format.timestamp.is_none()),
        _ => panic!("Expected SemVer config"),
    }
    assert!(version.timestamp.is_none());
}

#[test]
fn test_timestamp_config_with_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    write_test_config_with_timestamp(&file_path, (1, 0, 0, 0), "%Y-%m-%d");

    let version = Version::from_file(&file_path).unwrap();

    // Config should have the format string
    match &version.config {
        Config::SemVer(cfg) => {
            assert!(cfg.format.timestamp.is_some());
            assert_eq!(cfg.format.timestamp.as_ref().unwrap(), "%Y-%m-%d");
        },
        _ => panic!("Expected SemVer config"),
    }

    // Timestamp should be generated during from_file
    assert!(version.timestamp.is_some());
    let timestamp = version.timestamp.as_ref().unwrap();

    // Should match YYYY-MM-DD format
    assert_eq!(timestamp.len(), 10);
    assert!(timestamp.contains('-'));
}

#[test]
fn test_timestamp_iso8601_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    write_test_config_with_timestamp(&file_path, (1, 0, 0, 0), "%Y-%m-%dT%H:%M:%S%z");

    let version = Version::from_file(&file_path).unwrap();

    assert!(version.timestamp.is_some());
    let timestamp = version.timestamp.as_ref().unwrap();

    // Should contain ISO8601 format elements
    assert!(timestamp.contains('T'));
    assert!(timestamp.contains(':'));
}

#[test]
fn test_timestamp_custom_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    write_test_config_with_timestamp(&file_path, (1, 0, 0, 0), "%Y%m%d_%H%M%S");

    let version = Version::from_file(&file_path).unwrap();

    assert!(version.timestamp.is_some());
    let timestamp = version.timestamp.as_ref().unwrap();

    // Should match compact format YYYYMMDD_HHMMSS
    assert_eq!(timestamp.len(), 15); // 8 digits + 1 underscore + 6 digits
    assert!(timestamp.contains('_'));
}

#[test]
fn test_timestamp_updates_on_bump() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let content = r#"[semver.format]
prefix = "v"
delimiter = "."
timestamp = "%Y-%m-%d %H:%M:%S"

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
"#;
    fs::write(&file_path, content).unwrap();

    let mut version = Version::from_file(&file_path).unwrap();
    let original_timestamp = version.timestamp.clone();

    assert!(original_timestamp.is_some());

    // Sleep briefly to ensure timestamp difference
    std::thread::sleep(std::time::Duration::from_millis(1100));

    // Bump should update the timestamp
    version.bump(&BumpType::Point(PointType::Patch)).unwrap();

    assert!(version.timestamp.is_some());
    // Timestamps should be different (assuming they have at least second precision)
    assert_ne!(version.timestamp, original_timestamp);
}

#[test]
fn test_timestamp_in_c_header_output() {
    let _repo = create_temp_dir();
    let config_path = _repo.path().join("bump.toml");
    let output_path = _repo.path().join("version.h");

        let config_content = r#"[semver.format]
prefix = "v"
delimiter = "."
timestamp = "%Y-%m-%d"

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
        fs::write(&config_path, config_content).unwrap();

    let version = Version::from_file(&config_path).unwrap();

    crate::lang::output_file(
        &crate::lang::Language::C,
        &version,
        &output_path,
    )
    .unwrap();

    let header_content = fs::read_to_string(&output_path).unwrap();

    // Should contain VERSION_TIMESTAMP define
    assert!(header_content.contains("#define VERSION_TIMESTAMP"));
    assert!(header_content.contains(version.timestamp.as_ref().unwrap().as_str()));
}

#[test]
fn test_timestamp_not_in_c_header_when_none() {
    let _repo = create_temp_dir();
    let config_path = _repo.path().join("bump.toml");
    let output_path = _repo.path().join("version.h");

    let config_content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&config_path, config_content).unwrap();

    let version = Version::from_file(&config_path).unwrap();

    assert!(version.timestamp.is_none());

    crate::lang::output_file(
        &crate::lang::Language::C,
        &version,
        &output_path,
    )
    .unwrap();

    let header_content = fs::read_to_string(&output_path).unwrap();

    // Should NOT contain VERSION_TIMESTAMP define when timestamp is None
    assert!(!header_content.contains("#define VERSION_TIMESTAMP"));
}

#[test]
fn test_timestamp_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    let config = make_default_config(1, 2, 3, 0);

    let version = Version {
        prefix: "v".to_string(),
        timestamp: Some("2025-11-07 12:00:00 UTC".to_string()),
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 0,
        },
        path: file_path.clone(),
        config,
    };

    // Write to file
    version.file_init().unwrap();
    version.to_file().unwrap();

    // Read back from file
    let read_version = Version::from_file(&file_path).unwrap();

    // Config timestamp format should be preserved
    match &read_version.config {
        Config::SemVer(cfg) => {
            assert_eq!(
                cfg.format.timestamp,
                Some("%Y-%m-%d %H:%M:%S %Z".to_string())
            );
        },
        _ => panic!("Expected SemVer config"),
    }

    // Timestamp value should be generated (will be different from original)
    assert!(read_version.timestamp.is_some());
}

#[test]
fn test_timestamp_default_version() {
    let path = PathBuf::from("test.toml");
    let version = Version::default(&path);

    // Default version should have no timestamp configured
    match &version.config {
        Config::SemVer(cfg) => assert!(cfg.format.timestamp.is_some()),
        _ => panic!("Expected SemVer config"),
    }
    assert!(version.timestamp.is_none());
}

#[test]
fn test_timestamp_with_candidate_bump() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    write_test_config_with_timestamp(&file_path, (1, 0, 0, 0), "%Y%m%d");

    let mut version = Version::from_file(&file_path).unwrap();

    // Bump to candidate should also update timestamp
    version.bump(&BumpType::Candidate).unwrap();

    assert!(version.timestamp.is_some());
    match &version.version_type {
        VersionType::SemVer { candidate, .. } => assert_eq!(*candidate, 1),
        _ => panic!("Expected SemVer version type"),
    }

    // Timestamp should be 8 digits (YYYYMMDD)
    let timestamp = version.timestamp.as_ref().unwrap();
    assert_eq!(timestamp.len(), 8);
    assert!(timestamp.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_timestamp_with_release_bump() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    write_test_config_with_timestamp(&file_path, (1, 0, 0, 1), "%Y-%m-%d");

    let mut version = Version::from_file(&file_path).unwrap();

    match &version.version_type {
        VersionType::SemVer { candidate, .. } => assert_eq!(*candidate, 1),
        _ => panic!("Expected SemVer version type"),
    }

    // Release should update timestamp
    version.bump(&BumpType::Release).unwrap();

    assert!(version.timestamp.is_some());
    match &version.version_type {
        VersionType::SemVer { candidate, .. } => assert_eq!(*candidate, 0),
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn test_timestamp_human_readable_format() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    write_test_config_with_timestamp(&file_path, (1, 0, 0, 0), "%B %d, %Y");

    let version = Version::from_file(&file_path).unwrap();

    assert!(version.timestamp.is_some());
    let timestamp = version.timestamp.as_ref().unwrap();

    // Should contain month name and comma
    assert!(timestamp.contains(','));
    // Should contain a space
    assert!(timestamp.contains(' '));
}

#[test]
fn version_bump_major() {
    let config = make_default_config(1, 2, 3, 4);

    let mut version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    version.bump(&BumpType::Point(PointType::Major)).unwrap();

    assert_eq!(version.prefix, "v");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 2);
            assert_eq!(*minor, 0);
            assert_eq!(*patch, 0);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn version_bump_minor() {
    let config = make_default_config(1, 2, 3, 4);

    let mut version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    version.bump(&BumpType::Point(PointType::Minor)).unwrap();

    assert_eq!(version.prefix, "v");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 3);
            assert_eq!(*patch, 0);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn version_bump_patch() {
    let config = make_default_config(1, 2, 3, 4);

    let mut version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    version.bump(&BumpType::Point(PointType::Patch)).unwrap();

    assert_eq!(version.prefix, "v");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 2);
            assert_eq!(*patch, 4);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn version_bump_candidate() {
    let mut config = make_default_config(1, 2, 3, 4);
    match &mut config {
        Config::SemVer(cfg) => cfg.format.prefix = "prefix_".to_string(),
        _ => panic!("Expected SemVer config"),
    }

    let mut version = Version {
        prefix: "prefix_".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    version.bump(&BumpType::Candidate).unwrap();

    assert_eq!(version.prefix, "prefix_");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 2);
            assert_eq!(*patch, 3); // Patch is unchanged when incrementing existing candidate
            assert_eq!(*candidate, 5);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn version_bump_candidate_existing_value() {
    let config = make_default_config(1, 2, 3, 4);

    let mut version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    // Test candidate bump - should increment candidate
    version.bump(&BumpType::Candidate).unwrap();
    assert_eq!(version.prefix, "v");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1); // Unchanged
            assert_eq!(*minor, 2); // Unchanged  
            assert_eq!(*patch, 3); // Unchanged when incrementing existing candidate
            assert_eq!(*candidate, 5); // Incremented
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn version_bump_sequence() {
    let config = make_default_config(1, 0, 0, 0);

    let mut version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 0,
            patch: 0,
            candidate: 0,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    // Bump patch
    version.bump(&BumpType::Point(PointType::Patch)).unwrap();
    assert_eq!(version.prefix, "v");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 0);
            assert_eq!(*patch, 1);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }

    // Bump candidate (should bump minor when candidate is 0)
    version.bump(&BumpType::Candidate).unwrap();
    assert_eq!(version.prefix, "v");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 1); // Minor bumped because candidate was 0
            assert_eq!(*patch, 0); // Candidate bumps reset patch to 0
            assert_eq!(*candidate, 1);
        },
        _ => panic!("Expected SemVer version type"),
    }

    // Bump minor (should reset patch and candidate)
    version.bump(&BumpType::Point(PointType::Minor)).unwrap();
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 2); // Was 1, now bumped to 2
            assert_eq!(*patch, 0);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }

    // Bump major (should reset minor, patch and candidate)
    version.bump(&BumpType::Point(PointType::Major)).unwrap();
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 2);
            assert_eq!(*minor, 0);
            assert_eq!(*patch, 0);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn bump_types() {
    // Test that the enum variants exist and can be constructed
    let _major = BumpType::Point(PointType::Major);
    let _minor = BumpType::Point(PointType::Minor);
    let _patch = BumpType::Point(PointType::Patch);
    let _candidate = BumpType::Candidate;
    let _release = BumpType::Release;
    let _development = BumpType::Candidate;
}

#[test]
fn point_types() {
    // Test that the enum variants exist
    let _major = PointType::Major;
    let _minor = PointType::Minor;
    let _patch = PointType::Patch;
}

#[test]
fn version_bump_patch_with_candidate() {
    let config = make_default_config(1, 2, 3, 4);

    let mut version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    version.bump(&BumpType::Point(PointType::Patch)).unwrap();

    // Patch bump should increment patch and reset candidate
    assert_eq!(version.prefix, "v");
    match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1); // Unchanged
            assert_eq!(*minor, 2); // Unchanged
            assert_eq!(*patch, 4); // Incremented
            assert_eq!(*candidate, 0); // Reset
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn version_to_string_candidate_with_value() {
    let config = make_default_config(1, 2, 3, 4);

    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    // Candidate should show the -rc suffix
    assert_eq!(version.to_string(&BumpType::Candidate), "v1.2.3-rc4");
}

#[test]
fn version_to_string_none_tagged_without_candidate() {
    let config = make_default_config(1, 2, 3, 0);

    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 0,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    assert_eq!(
        version.to_string(&BumpType::Point(PointType::Patch)),
        "v1.2.3"
    );
}

#[test]
fn version_to_string_point_with_candidate() {
    let config = make_default_config(1, 2, 3, 4);

    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 1,
            minor: 2,
            patch: 3,
            candidate: 4,
        },
        path: PathBuf::from("test.toml"),
        config,
    };

    // Point release ignores candidate and shows just major.minor.patch
    assert_eq!(
        version.to_string(&BumpType::Point(PointType::Patch)),
        "v1.2.3"
    );
}

#[test]
fn version_preserves_comments_when_writing() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.toml");

    // Create a TOML file with comments
    let original_content = r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

[semver.format]
prefix = "v"
delimiter = "."

# NOTE: This section is modified by the bump command
[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"  # ["minor", "major", "patch"]
delimiter = "-rc"

# promotion strategies:
#  - git_sha ( 7 char sha1 of the current commit )
#  - branch ( append branch name )
#  - full ( <branch>_<sha1> )
[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&file_path, original_content).unwrap();

    // Load the version and modify it
    let mut version = Version::from_file(&file_path).unwrap();
    version.version_type = VersionType::SemVer {
        major: 2,
        minor: 0,
        patch: 0,
        candidate: 0,
    };

    // Write it back
    version.to_file().unwrap();

    // Read the file back and check that comments are preserved
    let updated_content = fs::read_to_string(&file_path).unwrap();

    // Check that comments are preserved
    assert!(updated_content.contains("# https://github.com/launchfirestorm/bump"));
    assert!(updated_content.contains("# NOTE: This section is modified by the bump command"));
    assert!(updated_content.contains("# promotion strategies:"));
    assert!(updated_content.contains("#  - git_sha ( 7 char sha1 of the current commit )"));
    assert!(updated_content.contains("#  - branch ( append branch name )"));
    assert!(updated_content.contains("#  - full ( <branch>_<sha1> )"));

    // Check that values are updated
    assert!(updated_content.contains("major = 2"));
    assert!(updated_content.contains("minor = 0"));
    assert!(updated_content.contains("patch = 0"));
}

#[test]
fn test_gen_command_c_output() {
    let _repo = create_temp_dir();
    let config_path = _repo.path().join("bump.toml");
    let output_path = _repo.path().join("version.h");

    // Create a test bump.toml file
    let config_content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Load version and generate C header
    let version = Version::from_file(&config_path).unwrap();
    crate::lang::output_file(
        &crate::lang::Language::C,
        &version,
        &output_path,
    )
    .unwrap();

    // Verify C header content
    let header_content = fs::read_to_string(&output_path).unwrap();
    assert!(header_content.contains("#define VERSION_PREFIX \"v\""));
    assert!(header_content.contains("#define VERSION_MAJOR 1"));
    assert!(header_content.contains("#define VERSION_MINOR 2"));
    assert!(header_content.contains("#define VERSION_PATCH 3"));
    assert!(header_content.contains("#define VERSION_CANDIDATE 0"));
    assert!(header_content.contains("#define VERSION_STRING \""));
    assert!(header_content.contains("v1.2.3"));
    assert!(header_content.contains("#ifndef BUMP_VERSION_H"));
    assert!(header_content.contains("#endif /* BUMP_VERSION_H */"));
    assert!(header_content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_gen_command_go_output() {
    let _repo = create_temp_dir();
    let config_path = _repo.path().join("bump.toml");
    let output_path = _repo.path().join("version.go");

    // Create a test bump.toml file
    let config_content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 2
minor = 1
patch = 0
candidate = 5

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "branch"
delimiter = "+"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Load version and generate Go file
    let version = Version::from_file(&config_path).unwrap();
    crate::lang::output_file(
        &crate::lang::Language::Go,
        &version,
        &output_path,
    )
    .unwrap();

    // Verify Go file content
    let go_content = fs::read_to_string(&output_path).unwrap();
    assert!(go_content.contains("package version"));
    assert!(go_content.contains("PREFIX    = \"v\""));
    assert!(go_content.contains("MAJOR     = 2"));
    assert!(go_content.contains("MINOR     = 1"));
    assert!(go_content.contains("PATCH     = 0"));
    assert!(go_content.contains("CANDIDATE = 5"));
    assert!(go_content.contains("STRING    = \""));
    assert!(go_content.contains("v2.1.0-rc5"));
    assert!(go_content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_gen_command_java_output() {
    let _repo = create_temp_dir();
    let config_path = _repo.path().join("bump.toml");
    let output_path = _repo.path().join("Version.java");

    // Create a test bump.toml file
    let config_content = r#"[semver.format]
prefix = "release-"
delimiter = "."

[semver.version]
major = 3
minor = 0
patch = 1
candidate = 0

[semver.candidate]
promotion = "major"
delimiter = "-beta"

[semver.development]
promotion = "full"
delimiter = "_"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Load version and generate Java file
    let version = Version::from_file(&config_path).unwrap();
    crate::lang::output_file(
        &crate::lang::Language::Java,
        &version,
        &output_path,
    )
    .unwrap();

    // Verify Java file content
    let java_content = fs::read_to_string(&output_path).unwrap();
    assert!(java_content.contains("public class Version"));
    assert!(java_content.contains("public static final String PREFIX = \"release-\";"));
    assert!(java_content.contains("public static final int MAJOR = 3;"));
    assert!(java_content.contains("public static final int MINOR = 0;"));
    assert!(java_content.contains("public static final int PATCH = 1;"));
    assert!(java_content.contains("public static final int CANDIDATE = 0;"));
    assert!(java_content.contains("public static final String STRING = \""));
    assert!(java_content.contains("release-3.0.1"));
    assert!(java_content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_gen_command_csharp_output() {
    let _repo = create_temp_dir();
    let config_path = _repo.path().join("bump.toml");
    let output_path = _repo.path().join("Version.cs");

    // Create a test bump.toml file
    let config_content = r#"[semver.format]
prefix = ""
delimiter = "."

[semver.version]
major = 0
minor = 5
patch = 12
candidate = 2

[semver.candidate]
promotion = "patch"
delimiter = "-alpha"

[semver.development]
promotion = "git_sha"
delimiter = "-dev"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Load version and generate C# file
    let version = Version::from_file(&config_path).unwrap();
    crate::lang::output_file(
        &crate::lang::Language::CSharp,
        &version,
        &output_path,
    )
    .unwrap();

    // Verify C# file content
    let csharp_content = fs::read_to_string(&output_path).unwrap();
    assert!(csharp_content.contains("public static class Version"));
    assert!(csharp_content.contains("public const string PREFIX = \"\";"));
    assert!(csharp_content.contains("public const int MAJOR = 0;"));
    assert!(csharp_content.contains("public const int MINOR = 5;"));
    assert!(csharp_content.contains("public const int PATCH = 12;"));
    assert!(csharp_content.contains("public const int CANDIDATE = 2;"));
    assert!(csharp_content.contains("public const string STRING = \""));
    assert!(csharp_content.contains("0.5.12-alpha2"));
    assert!(csharp_content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_development_suffix_strategies() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Test git_sha strategy
    let config_content_sha = r#"[semver.format]
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
"#;
    fs::write(&config_path, config_content_sha).unwrap();
    let version_sha = Version::from_file(&config_path).unwrap();

    // Test branch strategy
    let config_content_branch = r#"[semver.format]
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
promotion = "branch"
delimiter = "+"
"#;
    fs::write(&config_path, config_content_branch).unwrap();
    let version_branch = Version::from_file(&config_path).unwrap();

    // Test full strategy
    let config_content_full = r#"[semver.format]
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
promotion = "full"
delimiter = "+"
"#;
    fs::write(&config_path, config_content_full).unwrap();
    let version_full = Version::from_file(&config_path).unwrap();

    // Verify the promotion strategies are correctly configured
    match &version_sha.config {
        Config::SemVer(cfg) => assert_eq!(cfg.development.promotion, "git_sha"),
        _ => panic!("Expected SemVer config"),
    }
    match &version_branch.config {
        Config::SemVer(cfg) => assert_eq!(cfg.development.promotion, "branch"),
        _ => panic!("Expected SemVer config"),
    }
    match &version_full.config {
        Config::SemVer(cfg) => assert_eq!(cfg.development.promotion, "full"),
        _ => panic!("Expected SemVer config"),
    }

    // Verify delimiters
    match &version_sha.config {
        Config::SemVer(cfg) => assert_eq!(cfg.development.delimiter, "+"),
        _ => panic!("Expected SemVer config"),
    }
    match &version_branch.config {
        Config::SemVer(cfg) => assert_eq!(cfg.development.delimiter, "+"),
        _ => panic!("Expected SemVer config"),
    }
    match &version_full.config {
        Config::SemVer(cfg) => assert_eq!(cfg.development.delimiter, "+"),
        _ => panic!("Expected SemVer config"),
    }
}

#[test]
fn test_candidate_promotion_strategies() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Test minor promotion strategy (default)
    let config_content_minor = r#"[semver.format]
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
"#;
    fs::write(&config_path, config_content_minor).unwrap();
    let mut version_minor = Version::from_file(&config_path).unwrap();
    version_minor.bump(&BumpType::Candidate).unwrap();
    match &version_minor.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1);
            assert_eq!(*minor, 1); // Should be bumped
            assert_eq!(*patch, 0); // Should be reset
            assert_eq!(*candidate, 1);
        },
        _ => panic!("Expected SemVer version type"),
    }

    // Test major promotion strategy
    let config_content_major = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "major"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&config_path, config_content_major).unwrap();
    let mut version_major = Version::from_file(&config_path).unwrap();
    version_major.bump(&BumpType::Candidate).unwrap();
    match &version_major.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 2); // Should be bumped
            assert_eq!(*minor, 0); // Should be reset
            assert_eq!(*patch, 0); // Should be reset
            assert_eq!(*candidate, 1);
        },
        _ => panic!("Expected SemVer version type"),
    }

    // Test patch promotion strategy
    let config_content_patch = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "patch"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
    fs::write(&config_path, config_content_patch).unwrap();
    let mut version_patch = Version::from_file(&config_path).unwrap();
    version_patch.bump(&BumpType::Candidate).unwrap();
    match &version_patch.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 1); // Should be unchanged
            assert_eq!(*minor, 2); // Should be unchanged
            assert_eq!(*patch, 4); // Should be bumped
            assert_eq!(*candidate, 1);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

#[test]
fn test_multiple_output_files() {
    let _repo = create_temp_dir();
    let config_path = _repo.path().join("bump.toml");
    let output_path_1 = _repo.path().join("version1.h");
    let output_path_2 = _repo.path().join("include/version2.h");

        // Create a test bump.toml file
        let config_content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
        fs::write(&config_path, config_content).unwrap();
        let version = Version::from_file(&config_path).unwrap();

        // Create include directory
        fs::create_dir_all(output_path_2.parent().unwrap()).unwrap();

        // Generate multiple C headers
    crate::lang::output_file(
        &crate::lang::Language::C,
        &version,
        &output_path_1,
    )
    .unwrap();

    crate::lang::output_file(
        &crate::lang::Language::C,
        &version,
        &output_path_2,
    )
    .unwrap();

        // Verify both files exist and have correct content
        assert!(output_path_1.exists());
        assert!(output_path_2.exists());

        let content_1 = fs::read_to_string(&output_path_1).unwrap();
        let content_2 = fs::read_to_string(&output_path_2).unwrap();

        // Both should have the same version info
    for content in [&content_1, &content_2] {
        assert!(content.contains("#define VERSION_MAJOR 1"));
        assert!(content.contains("#define VERSION_MINOR 2"));
        assert!(content.contains("#define VERSION_PATCH 3"));
        assert!(content.contains("#define VERSION_STRING \""));
        assert!(content.contains("v1.2.3"));
    }
}

#[test]
fn test_git_branch_detection() {
    let _repo = create_temp_git_repo(false);
    let branch = get_git_branch().unwrap();
    assert!(!branch.is_empty(), "Branch name should not be empty");
}

#[test]
fn test_update_cargo_toml() {
    let _repo = create_temp_git_repo(false);
    let config_path = _repo.path().join("bump.toml");
    let cargo_path = _repo.path().join("Cargo.toml");

        // Create a test bump.toml file
        let config_content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 2
minor = 3
patch = 4
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
        write_bump_toml(&config_path, config_content);

        // Create a test Cargo.toml file with existing content
        let cargo_content = r#"[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

# This is a comment that should be preserved
[dependencies]
serde = "1.0"
"#;
        fs::write(&cargo_path, cargo_content).unwrap();

        // Load version and update Cargo.toml
        let version = Version::from_file(&config_path).unwrap();
    crate::update::cargo_toml(&version, &cargo_path).unwrap();

        // Verify Cargo.toml content
        let updated_content = fs::read_to_string(&cargo_path).unwrap();
    let expected_version = version
        .fully_qualified_string()
        .unwrap()
        .trim_start_matches('v')
        .to_string();

        // Version should be updated (without 'v' prefix)
    assert!(updated_content.contains(&format!(
        "version = \"{}\"",
        expected_version
    )));

        // Other fields should be preserved
        assert!(updated_content.contains("name = \"test-package\""));
        assert!(updated_content.contains("edition = \"2021\""));

        // Comments should be preserved
        assert!(updated_content.contains("# This is a comment that should be preserved"));

        // Dependencies should be preserved
    assert!(updated_content.contains("[dependencies]"));
    assert!(updated_content.contains("serde = \"1.0\""));
}

#[test]
fn test_update_cargo_toml_with_dev_suffix() {
    let _repo = create_temp_git_repo(false);
    let config_path = _repo.path().join("bump.toml");
    let cargo_path = _repo.path().join("Cargo.toml");

        // Create a test bump.toml file
        let config_content = r#"[semver.format]
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
"#;
        write_bump_toml(&config_path, config_content);

        // Create a test Cargo.toml file
        let cargo_content = r#"[package]
name = "my-crate"
version = "0.0.1"
edition = "2021"
"#;
        fs::write(&cargo_path, cargo_content).unwrap();

        // Load version and update Cargo.toml with development suffix
        let version = Version::from_file(&config_path).unwrap();
    crate::update::cargo_toml(&version, &cargo_path).unwrap();

        // Verify Cargo.toml content - should have version with build metadata
        let updated_content = fs::read_to_string(&cargo_path).unwrap();
    let expected_version = version
        .fully_qualified_string()
        .unwrap()
        .trim_start_matches('v')
        .to_string();
    assert!(updated_content.contains(&format!(
        "version = \"{}\"",
        expected_version
    )));
}

#[test]
fn test_fully_qualified_string_with_dev_suffix_when_untagged() {
    let _repo = create_temp_git_repo(false);
    let config_path = _repo.path().join("bump.toml");
    let short_sha = git_rev_parse_short_in(_repo.path());
    let config_content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
        write_bump_toml(&config_path, config_content);

    let version = Version::from_file(&config_path).unwrap();
    let expected = format!("v1.2.3+{}", short_sha);
    assert_eq!(
        version
            .fully_qualified_string()
            .unwrap(),
        expected
    );
}

#[test]
fn test_fully_qualified_string_without_dev_suffix_when_tagged() {
    let _repo = create_temp_git_repo(true);
    let config_path = _repo.path().join("bump.toml");
    let config_content = r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = 1
minor = 2
patch = 3
candidate = 0

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#;
        write_bump_toml(&config_path, config_content);

    let version = Version::from_file(&config_path).unwrap();
    assert_eq!(
        version
            .fully_qualified_string()
            .unwrap(),
        "v1.2.3"
    );
}

#[test]
fn test_update_cargo_toml_missing_package_section() {
    let _repo = create_temp_git_repo(false);
    let config_path = _repo.path().join("bump.toml");
    let cargo_path = _repo.path().join("Cargo.toml");

    // Create a test bump.toml file
    let config_content = r#"[semver.format]
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
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create a Cargo.toml without [package] section
    let cargo_content = r#"[dependencies]
serde = "1.0"
"#;
    fs::write(&cargo_path, cargo_content).unwrap();

    // Load version and try to update - should fail
    let version = Version::from_file(&config_path).unwrap();
    let result = crate::update::cargo_toml(&version, &cargo_path);

    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::ParseError(msg) => {
            assert!(msg.contains("no [package] section"));
        }
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_init_semver_creates_proper_structure() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("bump.toml");

    // Create a new SemVer version by initializing
    let version = Version {
        prefix: "v".to_string(),
        timestamp: None,
        version_type: VersionType::SemVer {
            major: 0,
            minor: 1,
            patch: 0,
            candidate: 0,
        },
        path: config_path.clone(),
        config: crate::version::default_semver_config("v".to_string(), 0, 1, 0, 0),
    };
    
    version.file_init().unwrap();

    // Verify file was created
    assert!(config_path.exists());

    // Read the file and verify structure
    let content = fs::read_to_string(&config_path).unwrap();

    // Check for header
    assert!(content.contains("https://github.com/launchfirestorm/bump"));

    // Check for [semver.format] section
    assert!(content.contains("[semver.format]"));
    assert!(content.contains("prefix = \"v\""));

    // Check version values
    assert!(content.contains("[semver.version]"));
    assert!(content.contains("major = 0"));
    assert!(content.contains("minor = 1"));
    assert!(content.contains("patch = 0"));
    assert!(content.contains("candidate = 0"));

    // Check for candidate section with comments
    assert!(content.contains("[semver.candidate]"));
    assert!(content.contains("promotion = \"minor\""));
    assert!(content.contains("delimiter = \"-rc\""));
    assert!(content.contains("Candidate promotion strategies:"));

    // Check for development section with comments
    assert!(content.contains("[semver.development]"));
    assert!(content.contains("promotion = \"git_sha\""));
    assert!(content.contains("delimiter = \"+\""));
    assert!(content.contains("Development suffix strategies:"));
    assert!(content.contains("git_sha"));
    assert!(content.contains("branch"));
    assert!(content.contains("full"));

    // Verify it can be read back
    let read_version = Version::from_file(&config_path).unwrap();
    match &read_version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            assert_eq!(*major, 0);
            assert_eq!(*minor, 1);
            assert_eq!(*patch, 0);
            assert_eq!(*candidate, 0);
        },
        _ => panic!("Expected SemVer version type"),
    }
}

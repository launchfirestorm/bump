use super::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_version_default() {
    let path = PathBuf::from("test.bumpfile");
    let version = Version::default(&path);

    assert_eq!(version.major, 0);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, String::new());
    assert_eq!(version.path, path);
}

#[test]
fn test_version_from_file_valid() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=1\nMINOR=2\nPATCH=3\nCANDIDATE=0\nCOMMIT=abc1234\n";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "abc1234");
    assert_eq!(version.path, file_path);
}

#[test]
fn test_version_from_file_invalid_major() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=invalid\nMINOR=2\nPATCH=3\nCANDIDATE=0\nCOMMIT=abc1234\n";
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::ParseError(field) => assert_eq!(field, "MAJOR"),
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_version_from_file_invalid_minor() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=1\nMINOR=invalid\nPATCH=3\nCANDIDATE=0\nCOMMIT=abc1234\n";
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::ParseError(field) => assert_eq!(field, "MINOR"),
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_version_from_file_invalid_patch() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=1\nMINOR=2\nPATCH=invalid\nCANDIDATE=0\nCOMMIT=abc1234\n";
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::ParseError(field) => assert_eq!(field, "PATCH"),
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_version_from_file_invalid_candidate() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=1\nMINOR=2\nPATCH=3\nCANDIDATE=invalid\nCOMMIT=abc1234\n";
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::ParseError(field) => assert_eq!(field, "CANDIDATE"),
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_version_from_file_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nonexistent.bumpfile");

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::IoError(_) => (), // Expected
        _ => panic!("Expected IoError"),
    }
}

#[test]
fn test_version_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "abc1234".to_string(),
        path: file_path.clone(),
    };

    version.to_file().unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("MAJOR=1"));
    assert!(content.contains("MINOR=2"));
    assert!(content.contains("PATCH=3"));
    assert!(content.contains("CANDIDATE=4"));
    assert!(content.contains("COMMIT=abc1234"));
    assert!(content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_version_to_string_point() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 0,
        commit: "abc1234".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    let version_string = version.to_string(&BumpType::Point(PointType::PATCH));
    assert_eq!(version_string, "1.2.3");
}

#[test]
fn test_version_to_string_candidate() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "abc1234".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    let version_string = version.to_string(&BumpType::Candidate);
    assert_eq!(version_string, "1.2.3-rc4");
}

#[test]
fn test_version_to_string_development() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 0,
        commit: "abc1234".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    let version_string = version.to_string(&BumpType::Development);
    assert_eq!(version_string, "1.2.3+abc1234");
}

#[test]
fn test_version_to_header() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "abc1234".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    let header = version.to_header(&BumpType::Point(PointType::PATCH));

    assert!(header.contains("#define VERSION_MAJOR 1"));
    assert!(header.contains("#define VERSION_MINOR 2"));
    assert!(header.contains("#define VERSION_PATCH 3"));
    assert!(header.contains("#define VERSION_CANDIDATE 4"));
    assert!(header.contains("#define VERSION_COMMIT abc1234"));
    assert!(header.contains("#define VERSION_STRING \"1.2.3\""));
    assert!(header.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_resolve_path_absolute() {
    let absolute_path = if cfg!(windows) {
        "C:\\test\\path"
    } else {
        "/test/path"
    };

    let resolved = resolve_path(absolute_path);
    assert_eq!(resolved, PathBuf::from(absolute_path));
}

#[test]
fn test_resolve_path_relative() {
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
fn test_bump_error_display() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let bump_error = BumpError::IoError(io_error);

    let display = format!("{}", bump_error);
    assert!(display.contains("I/O error"));

    let parse_error = BumpError::ParseError("MAJOR".to_string());
    let display = format!("{}", parse_error);
    assert!(display.contains("Invalid MAJOR value"));

    let logic_error = BumpError::LogicError("Test error".to_string());
    let display = format!("{}", logic_error);
    assert!(display.contains("Error: Test error"));
}

#[test]
fn test_bump_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let bump_error: BumpError = io_error.into();

    match bump_error {
        BumpError::IoError(_) => (), // Expected
        _ => panic!("Expected IoError"),
    }
}

#[test]
fn test_version_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let original_version = Version {
        major: 5,
        minor: 10,
        patch: 15,
        candidate: 2,
        commit: "def5678".to_string(),
        path: file_path.clone(),
    };

    // Write to file
    original_version.to_file().unwrap();

    // Read from file
    let read_version = Version::from_file(&file_path).unwrap();

    assert_eq!(original_version.major, read_version.major);
    assert_eq!(original_version.minor, read_version.minor);
    assert_eq!(original_version.patch, read_version.patch);
    assert_eq!(original_version.candidate, read_version.candidate);
    assert_eq!(original_version.commit, read_version.commit);
    assert_eq!(original_version.path, read_version.path);
}

#[test]
fn test_version_file_with_comments() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "# This is a comment\nMAJOR=1\n# Another comment\nMINOR=2\nPATCH=3\nCANDIDATE=0\nCOMMIT=abc1234\n# End comment";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "abc1234");
}

#[test]
fn test_version_file_with_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR= 1 \nMINOR= 2 \nPATCH= 3 \nCANDIDATE= 0 \nCOMMIT= abc1234 \n";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "abc1234");
}

#[test]
fn test_get_commit_sha() {
    match get_commit_sha() {
        Ok(commit_sha) => {
            println!("Commit SHA: {}", commit_sha);
            assert!(!commit_sha.is_empty(), "Commit SHA should not be empty");
            assert_eq!(commit_sha.len(), 7, "Commit SHA should be 7 characters long");
            assert!(commit_sha.chars().all(|c| c.is_ascii_hexdigit()), "Commit SHA should only contain hex digits");
        },
        Err(e) => {
            println!("Git command failed (expected in some environments): {}", e);
            // Don't fail the test if we're not in a git repo or git isn't available
            // This makes the test more robust for CI/CD environments
        }
    }
}

// Note: The following tests for the bump() method cannot be easily tested
// without mocking the git command, as they depend on get_commit_sha().
// In a real testing environment, you would need to either:
// 1. Mock the get_commit_sha function
// 2. Ensure you're in a git repository with commits
// 3. Skip the bump tests when git is not available

#[test]
fn test_bump_types() {
    // Test that the enum variants exist and can be constructed
    let _major = BumpType::Point(PointType::MAJOR);
    let _minor = BumpType::Point(PointType::MINOR);
    let _patch = BumpType::Point(PointType::PATCH);
    let _candidate = BumpType::Candidate;
    let _development = BumpType::Development;
}

#[test]
fn test_point_types() {
    // Test that the enum variants exist
    let _major = PointType::MAJOR;
    let _minor = PointType::MINOR;
    let _patch = PointType::PATCH;
}
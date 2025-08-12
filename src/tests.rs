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
        commit: "abc1234".to_string(), // Actual commit SHA for development
        path: PathBuf::from("test.bumpfile"),
    };

    let version_string = version.to_string(&BumpType::Development);
    assert_eq!(version_string, "1.2.3+abc1234");
}

#[test]
fn test_version_to_string_development_tagged() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 0,
        commit: "tagged".to_string(), // Tagged releases
        path: PathBuf::from("test.bumpfile"),
    };

    let version_string = version.to_string(&BumpType::Development);
    assert_eq!(version_string, "1.2.3+tagged");
}

#[test]
fn test_version_to_header() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "tagged".to_string(), // Point releases use "tagged"
        path: PathBuf::from("test.bumpfile"),
    };

    let header = version.to_header(&BumpType::Point(PointType::PATCH));

    assert!(header.contains("#define VERSION_MAJOR 1"));
    assert!(header.contains("#define VERSION_MINOR 2"));
    assert!(header.contains("#define VERSION_PATCH 3"));
    assert!(header.contains("#define VERSION_CANDIDATE 4"));
    assert!(header.contains("#define VERSION_COMMIT tagged"));
    assert!(header.contains("#define VERSION_STRING \"1.2.3\""));
    assert!(header.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_version_to_header_development() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 0,
        commit: "abc1234".to_string(), // Development builds use actual commit SHA
        path: PathBuf::from("test.bumpfile"),
    };

    let header = version.to_header(&BumpType::Development);

    assert!(header.contains("#define VERSION_MAJOR 1"));
    assert!(header.contains("#define VERSION_MINOR 2"));
    assert!(header.contains("#define VERSION_PATCH 3"));
    assert!(header.contains("#define VERSION_CANDIDATE 0"));
    assert!(header.contains("#define VERSION_COMMIT abc1234"));
    assert!(header.contains("#define VERSION_STRING \"1.2.3+abc1234\""));
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
        commit: "tagged".to_string(), // Use "tagged" for consistency
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

// Note: The following tests for the bump() method can now be tested
// because point and candidate bumps no longer depend on git.
// Only development bumps require git access.

#[test]
fn test_version_bump_major() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "abc1234".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Point(PointType::MAJOR)).unwrap();

    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "tagged");
}

#[test]
fn test_version_bump_minor() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "abc1234".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Point(PointType::MINOR)).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 3);
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "tagged");
}

#[test]
fn test_version_bump_patch() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "abc1234".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Point(PointType::PATCH)).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 4);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "tagged");
}

#[test]
fn test_version_bump_candidate() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "abc1234".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Candidate).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.candidate, 5);
    assert_eq!(version.commit, "tagged");
}

#[test]
fn test_version_bump_development() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        commit: "oldcommit".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    // This test will only work in a git repository
    match version.bump(&BumpType::Development) {
        Ok(()) => {
            // If successful, the commit should be updated to the current git SHA
            assert_eq!(version.major, 1); // Unchanged
            assert_eq!(version.minor, 2); // Unchanged
            assert_eq!(version.patch, 3); // Unchanged
            assert_eq!(version.candidate, 4); // Unchanged
            assert_ne!(version.commit, "oldcommit"); // Should be updated
            assert_eq!(version.commit.len(), 7); // Should be 7-char SHA
        },
        Err(_) => {
            // Expected if not in a git repository
            println!("Development bump failed (expected if not in git repo)");
        }
    }
}

#[test]
fn test_version_bump_sequence() {
    let mut version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        candidate: 0,
        commit: "initial".to_string(),
        path: PathBuf::from("test.bumpfile"),
    };

    // Bump patch
    version.bump(&BumpType::Point(PointType::PATCH)).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 1);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "tagged");

    // Bump candidate
    version.bump(&BumpType::Candidate).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 1);
    assert_eq!(version.candidate, 1);
    assert_eq!(version.commit, "tagged");

    // Bump minor (should reset patch and candidate)
    version.bump(&BumpType::Point(PointType::MINOR)).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "tagged");

    // Bump major (should reset minor, patch and candidate)
    version.bump(&BumpType::Point(PointType::MAJOR)).unwrap();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.commit, "tagged");
}

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
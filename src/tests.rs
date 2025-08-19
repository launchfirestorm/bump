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
    assert_eq!(version.path, path);
}

#[test]
fn test_is_git_repository() {
    // This test will pass or fail depending on whether we're in a git repo
    // Just test that the function doesn't panic
    let _ = is_git_repository();
}

#[test]
fn test_get_git_tag_non_git_repo() {
    // This should fail if we're not in a git repo or not on a tagged commit
    match get_git_tag() {
        Ok(_) => {
            // If we're on a tagged commit, that's fine
        }
        Err(BumpError::Git(_)) => {
            // Expected if not in git repo or not on tagged commit
        }
        Err(_) => panic!("Unexpected error type"),
    }
}

#[test]
fn test_version_from_file_valid() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=1\nMINOR=2\nPATCH=3\nCANDIDATE=0\n";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.candidate, 0);
    assert_eq!(version.path, file_path);
}

#[test]
fn test_version_from_file_invalid_major() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=invalid\nMINOR=2\nPATCH=3\nCANDIDATE=0\n";
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

    let content = "MAJOR=1\nMINOR=invalid\nPATCH=3\nCANDIDATE=0\n";
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

    let content = "MAJOR=1\nMINOR=2\nPATCH=invalid\nCANDIDATE=0\n";
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

    let content = "MAJOR=1\nMINOR=2\nPATCH=3\nCANDIDATE=invalid\n";
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
        path: file_path.clone(),
    };

    version.to_file().unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("MAJOR=1"));
    assert!(content.contains("MINOR=2"));
    assert!(content.contains("PATCH=3"));
    assert!(content.contains("CANDIDATE=4"));
    assert!(content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_version_to_string_point() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 0,
        path: PathBuf::from("test.bumpfile"),
    };

    let version_string = version.to_string(&BumpType::Point(PointType::Patch));
    assert_eq!(version_string, "1.2.3");
}

#[test]
fn test_version_to_string_candidate() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    let version_string = version.to_string(&BumpType::Candidate);
    assert_eq!(version_string, "1.2.3-rc4");
}

#[test]
fn test_version_to_header() {
    let temp_dir = TempDir::new().unwrap();
    let header_path = temp_dir.path().join("version.h");

    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    crate::lang::output_file(
        &crate::lang::Language::C,
        &version,
        "1.2.3-rc4",
        &header_path,
    )
    .unwrap();

    let header_content = fs::read_to_string(&header_path).unwrap();
    assert!(header_content.contains("#define VERSION_MAJOR 1"));
    assert!(header_content.contains("#define VERSION_MINOR 2"));
    assert!(header_content.contains("#define VERSION_PATCH 3"));
    assert!(header_content.contains("#define VERSION_CANDIDATE 4"));
    assert!(header_content.contains("#define VERSION_STRING \"1.2.3-rc4\""));
    assert!(header_content.contains("https://github.com/launchfirestorm/bump"));
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

    let display = format!("{bump_error}");
    assert!(display.contains("I/O error"));

    let parse_error = BumpError::ParseError("MAJOR".to_string());
    let display = format!("{parse_error}");
    assert!(display.contains("Invalid MAJOR value"));

    let logic_error = BumpError::LogicError("Test error".to_string());
    let display = format!("{logic_error}");
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

    assert_eq!(original_version.path, read_version.path);
}

#[test]
fn test_version_file_with_comments() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "# This is a comment\nMAJOR=1\n# Another comment\nMINOR=2\nPATCH=3\nCANDIDATE=0\n# End comment";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.candidate, 0);
}

#[test]
fn test_version_file_with_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR= 1 \nMINOR= 2 \nPATCH= 3 \nCANDIDATE= 0 \n";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.candidate, 0);
}

#[test]
fn test_get_git_commit_sha() {
    match get_git_commit_sha() {
        Ok(commit_sha) => {
            println!("Commit SHA: {commit_sha}");
            assert!(!commit_sha.is_empty(), "Commit SHA should not be empty");
            assert_eq!(
                commit_sha.len(),
                7,
                "Commit SHA should be 7 characters long"
            );
            assert!(
                commit_sha.chars().all(|c| c.is_ascii_hexdigit()),
                "Commit SHA should only contain hex digits"
            );
        }
        Err(e) => {
            println!("Git command failed (expected in some environments): {e}");
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
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Point(PointType::Major)).unwrap();

    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);
}

#[test]
fn test_version_bump_minor() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Point(PointType::Minor)).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 3);
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);
}

#[test]
fn test_version_bump_patch() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Point(PointType::Patch)).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 4);
    assert_eq!(version.candidate, 0);
}

#[test]
fn test_version_bump_candidate() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Candidate).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 0); // Candidate bumps reset patch to 0
    assert_eq!(version.candidate, 5);
}

#[test]
fn test_version_bump_candidate_existing_value() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    // Test candidate bump - should increment candidate
    version.bump(&BumpType::Candidate).unwrap();
    assert_eq!(version.major, 1); // Unchanged
    assert_eq!(version.minor, 2); // Unchanged  
    assert_eq!(version.patch, 0); // Reset to 0
    assert_eq!(version.candidate, 5); // Incremented
}

#[test]
fn test_version_bump_sequence() {
    let mut version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        candidate: 0,
        path: PathBuf::from("test.bumpfile"),
    };

    // Bump patch
    version.bump(&BumpType::Point(PointType::Patch)).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 1);
    assert_eq!(version.candidate, 0);

    // Bump candidate (should bump minor when candidate is 0)
    version.bump(&BumpType::Candidate).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 1); // Minor bumped because candidate was 0
    assert_eq!(version.patch, 0); // Candidate bumps reset patch to 0
    assert_eq!(version.candidate, 1);

    // Bump minor (should reset patch and candidate)
    version.bump(&BumpType::Point(PointType::Minor)).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2); // Was 1, now bumped to 2
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);

    // Bump major (should reset minor, patch and candidate)
    version.bump(&BumpType::Point(PointType::Major)).unwrap();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    assert_eq!(version.candidate, 0);
}

#[test]
fn test_bump_types() {
    // Test that the enum variants exist and can be constructed
    let _major = BumpType::Point(PointType::Major);
    let _minor = BumpType::Point(PointType::Minor);
    let _patch = BumpType::Point(PointType::Patch);
    let _candidate = BumpType::Candidate;
    let _release = BumpType::Release;
    let _development = BumpType::Candidate;
}

#[test]
fn test_point_types() {
    // Test that the enum variants exist
    let _major = PointType::Major;
    let _minor = PointType::Minor;
    let _patch = PointType::Patch;
}

#[test]
fn test_version_bump_patch_with_candidate() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(&BumpType::Point(PointType::Patch)).unwrap();

    // Patch bump should increment patch and reset candidate
    assert_eq!(version.major, 1); // Unchanged
    assert_eq!(version.minor, 2); // Unchanged
    assert_eq!(version.patch, 4); // Incremented
    assert_eq!(version.candidate, 0); // Reset
}

#[test]
fn test_version_to_string_candidate_with_value() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    // Candidate should show the -rc suffix
    assert_eq!(version.to_string(&BumpType::Candidate), "1.2.3-rc4");
}

#[test]
fn test_version_to_string_none_tagged_without_candidate() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 0,
        path: PathBuf::from("test.bumpfile"),
    };

    assert_eq!(
        version.to_string(&BumpType::Point(PointType::Patch)),
        "1.2.3"
    );
}

#[test]
fn test_version_to_string_point_with_candidate() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        candidate: 4,
        path: PathBuf::from("test.bumpfile"),
    };

    // Point release ignores candidate and shows just major.minor.patch
    assert_eq!(
        version.to_string(&BumpType::Point(PointType::Patch)),
        "1.2.3"
    );
}

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
    assert_eq!(version.path, path);
}

#[test]
fn test_version_from_file_valid() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=1\nMINOR=2\nPATCH=3\n";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.path, file_path);
}

#[test]
fn test_version_from_file_invalid_major() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR=invalid\nMINOR=2\nPATCH=3\n";
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

    let content = "MAJOR=1\nMINOR=invalid\nPATCH=3\n";
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

    let content = "MAJOR=1\nMINOR=2\nPATCH=invalid\n";
    fs::write(&file_path, content).unwrap();

    let result = Version::from_file(&file_path);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::ParseError(field) => assert_eq!(field, "PATCH"),
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
        path: file_path.clone(),
    };

    version.to_file().unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("MAJOR=1"));
    assert!(content.contains("MINOR=2"));
    assert!(content.contains("PATCH=3"));
    assert!(content.contains("https://github.com/launchfirestorm/bump"));
}

#[test]
fn test_version_bump_patch() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(false, false, true).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 4);
}

#[test]
fn test_version_bump_minor() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(false, true, false).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 3);
    assert_eq!(version.patch, 0);
}

#[test]
fn test_version_bump_major() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        path: PathBuf::from("test.bumpfile"),
    };

    version.bump(true, false, false).unwrap();

    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
}

#[test]
fn test_version_bump_nothing() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        path: PathBuf::from("test.bumpfile"),
    };

    let result = version.bump(false, false, false);
    assert!(result.is_err());
    match result.unwrap_err() {
        BumpError::LogicError(msg) => {
            assert!(msg.contains("Nothing to bump"));
        }
        _ => panic!("Expected LogicError"),
    }
}

#[test]
fn test_version_to_header() {
    let version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        path: PathBuf::from("test.bumpfile"),
    };

    let header = version.to_header();

    assert!(header.contains("#define VERSION_MAJOR 1"));
    assert!(header.contains("#define VERSION_MINOR 2"));
    assert!(header.contains("#define VERSION_PATCH 3"));
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
        path: file_path.clone(),
    };

    // Write to file
    original_version.to_file().unwrap();

    // Read from file
    let read_version = Version::from_file(&file_path).unwrap();

    assert_eq!(original_version.major, read_version.major);
    assert_eq!(original_version.minor, read_version.minor);
    assert_eq!(original_version.patch, read_version.patch);
    assert_eq!(original_version.path, read_version.path);
}

#[test]
fn test_version_bump_sequence() {
    let mut version = Version {
        major: 1,
        minor: 0,
        patch: 0,
        path: PathBuf::from("test.bumpfile"),
    };

    // Bump patch
    version.bump(false, false, true).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 1);

    // Bump minor
    version.bump(false, true, false).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 0);

    // Bump major
    version.bump(true, false, false).unwrap();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);

    // Bump patch again
    version.bump(false, false, true).unwrap();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 1);
}

#[test]
fn test_version_file_with_comments() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content =
        "# This is a comment\nMAJOR=1\n# Another comment\nMINOR=2\nPATCH=3\n# End comment";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
}

#[test]
fn test_version_file_with_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("version.bumpfile");

    let content = "MAJOR= 1 \nMINOR= 2 \nPATCH= 3 \n";
    fs::write(&file_path, content).unwrap();

    let version = Version::from_file(&file_path).unwrap();

    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
}

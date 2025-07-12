use crate::{Version, VersionError};
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_version_default() {
    let version = Version::default();
    assert_eq!(version.major, 0);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 0);
}

#[test]
fn test_version_from_file() {
    let file_path = "test_version_from_file";
    fs::write(file_path, "MAJOR=1\nMINOR=2\nPATCH=3\n").unwrap();
    let version = Version::from_file(Path::new(file_path)).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    
    fs::write(file_path, "# Comment\nMAJOR=4\nSomething else\nMINOR=5\nPATCH=6\n").unwrap();
    let version = Version::from_file(Path::new(file_path)).unwrap();
    assert_eq!(version.major, 4);
    assert_eq!(version.minor, 5);
    assert_eq!(version.patch, 6);
    
    let missing_path = "nonexistent_file";
    let result = Version::from_file(Path::new(missing_path));
    assert!(matches!(result, Err(VersionError::IoError(_))));
    
    fs::write(file_path, "MAJOR=invalid\nMINOR=2\nPATCH=3\n").unwrap();
    let result = Version::from_file(Path::new(file_path));
    assert!(matches!(result, Err(VersionError::ParseError(s)) if s == "MAJOR"));
    
    fs::write(file_path, "MAJOR=1\nMINOR=invalid\nPATCH=3\n").unwrap();
    let result = Version::from_file(Path::new(file_path));
    assert!(matches!(result, Err(VersionError::ParseError(s)) if s == "MINOR"));
    
    fs::write(file_path, "MAJOR=1\nMINOR=2\nPATCH=invalid\n").unwrap();
    let result = Version::from_file(Path::new(file_path));
    assert!(matches!(result, Err(VersionError::ParseError(s)) if s == "PATCH"));
    
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_version_to_file() {
    let file_path = "test_version_to_file";
    
    let version = Version {
        major: 2,
        minor: 3,
        patch: 4
    };
    
    version.to_file(Path::new(file_path)).unwrap();
    
    let content = fs::read_to_string(file_path).unwrap();
    assert!(content.contains("MAJOR=2"));
    assert!(content.contains("MINOR=3"));
    assert!(content.contains("PATCH=4"));
    
    // Clean up
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_version_bump() {
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3
    };
    version.bump(true, false, false);
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
    
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3
    };
    version.bump(false, true, false);
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 3);
    assert_eq!(version.patch, 0);
    
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3
    };
    version.bump(false, false, true);
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 4);
}

#[test]
fn test_version_to_header() {
    let version = Version {
        major: 2,
        minor: 3,
        patch: 4
    };
    
    let header = version.to_header();
    assert!(header.contains("#define VERSION_MAJOR 2"));
    assert!(header.contains("#define VERSION_MINOR 3"));
    assert!(header.contains("#define VERSION_PATCH 4"));
}

// Integration tests using direct process execution
// These tests require the binary to be built first!
// Run with: cargo build && cargo test
// #[test]
// fn test_integration_basic_workflow() {
//     let version_file = "version";
//     let header_file = "test_integration_header.h";
    
//     let _ = fs::remove_file(version_file);
//     let _ = fs::remove_file(header_file);
    
//     fs::write(version_file, "MAJOR=1\nMINOR=2\nPATCH=3\n").unwrap();
    
//     let bin_path = get_binary_path();
    
//     let output = Command::new(&bin_path)
//         .args(["--print"])
//         .output()
//         .expect("Failed to execute command");
    
//     assert!(output.status.success());
//     let stdout = String::from_utf8_lossy(&output.stdout);
//     assert!(stdout.contains("Found version: 1.2.3"));
    
//     let output = Command::new(&bin_path)
//         .args(["--patch"])
//         .output()
//         .expect("Failed to execute command");
    
//     assert!(output.status.success());
//     let stdout = String::from_utf8_lossy(&output.stdout);
//     assert!(stdout.contains("Version bumped to 1.2.4"));
    
//     let content = fs::read_to_string(version_file).unwrap();
//     assert!(content.contains("MAJOR=1"));
//     assert!(content.contains("MINOR=2"));
//     assert!(content.contains("PATCH=4"));
    
//     let output = Command::new(&bin_path)
//         .args(["--minor", "--output-file", header_file])
//         .output()
//         .expect("Failed to execute command");
    
//     assert!(output.status.success());
    
//     let content = fs::read_to_string(version_file).unwrap();
//     assert!(content.contains("MAJOR=1"));
//     assert!(content.contains("MINOR=3"));
//     assert!(content.contains("PATCH=0"));
    
//     let content = fs::read_to_string(header_file).unwrap();
//     assert!(content.contains("#define VERSION_MAJOR 1"));
//     assert!(content.contains("#define VERSION_MINOR 3"));
//     assert!(content.contains("#define VERSION_PATCH 0"));
    
//     let output = Command::new(&bin_path)
//         .args(["--major"])
//         .output()
//         .expect("Failed to execute command");
    
//     assert!(output.status.success());
    
//     let content = fs::read_to_string(version_file).unwrap();
//     assert!(content.contains("MAJOR=2"));
//     assert!(content.contains("MINOR=0"));
//     assert!(content.contains("PATCH=0"));
    
//     let _ = fs::remove_file(version_file);
//     let _ = fs::remove_file(header_file);
// }

#[test]
fn test_integration_nonexistent_file() {
    let version_file = "version";
    
    if Path::new(version_file).exists() {
        fs::remove_file(version_file).unwrap();
    }
    
    let bin_path = get_binary_path();
    
    let output = Command::new(&bin_path)
        .args(["--patch"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Creating a new one with default version"));
    
    assert!(Path::new(version_file).exists());
    let content = fs::read_to_string(version_file).unwrap();
    assert!(content.contains("MAJOR=0"));
    assert!(content.contains("MINOR=1"));
    assert!(content.contains("PATCH=1")); // Default is 0.1.0, patch bump makes it 0.1.1
    
    fs::remove_file(version_file).unwrap();
}

#[test]
fn test_integration_no_flags() {
    let version_file = "version";
    fs::write(version_file, "MAJOR=1\nMINOR=2\nPATCH=3\n").unwrap();
    
    let bin_path = get_binary_path();
    
    let output = Command::new(&bin_path)
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Must provide one of --major, --minor, or --patch"));
    
    let _ = fs::remove_file(version_file);
}

fn get_binary_path() -> String {
    let debug_path = "target/debug/bump";
    let release_path = "target/release/bump";
    let musl_debug_path = "target/x86_64-unknown-linux-musl/debug/bump";
    let musl_release_path = "target/x86_64-unknown-linux-musl/release/bump";
    
    if Path::new(debug_path).exists() {
        return debug_path.to_string();
    } else if Path::new(release_path).exists() {
        return release_path.to_string();
    } else if Path::new(musl_debug_path).exists() {
        return musl_debug_path.to_string();
    } else if Path::new(musl_release_path).exists() {
        return musl_release_path.to_string();
    } else {
        panic!("Could not find binary. Please run 'cargo build' before running the tests.");
    }
}

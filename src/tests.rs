use crate::{Version, VersionError};
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_version_default() {
    let test_path = Path::new("test_default");
    let version = Version::default(test_path);
    assert_eq!(version.major, 0);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 0);
    assert_eq!(version.path, test_path);
}

#[test]
fn test_version_from_file() {
    let file_path = "test_version_from_file";
    fs::write(file_path, "MAJOR=1\nMINOR=2\nPATCH=3\n").unwrap();
    let version = Version::from_file(Path::new(file_path)).unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(version.path, Path::new(file_path));

    fs::write(
        file_path,
        "# Comment\nMAJOR=4\nSomething else\nMINOR=5\nPATCH=6\n",
    )
    .unwrap();
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
        patch: 4,
        path: Path::new(file_path).to_path_buf(),
    };

    version.to_file().unwrap();

    let content = fs::read_to_string(file_path).unwrap();
    assert!(content.contains("MAJOR=2"));
    assert!(content.contains("MINOR=3"));
    assert!(content.contains("PATCH=4"));

    // Clean up
    fs::remove_file(file_path).unwrap();
}

#[test]
fn test_version_bump() {
    let test_path = Path::new("test_bump");
    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        path: test_path.to_path_buf(),
    };
    version.bump(true, false, false);
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);

    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        path: test_path.to_path_buf(),
    };
    version.bump(false, true, false);
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 3);
    assert_eq!(version.patch, 0);

    let mut version = Version {
        major: 1,
        minor: 2,
        patch: 3,
        path: test_path.to_path_buf(),
    };
    version.bump(false, false, true);
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 4);
}

#[test]
fn test_version_to_header() {
    let test_path = Path::new("test_header");
    let version = Version {
        major: 2,
        minor: 3,
        patch: 4,
        path: test_path.to_path_buf(),
    };

    let header = version.to_header();
    assert!(header.contains("#define VERSION_MAJOR 2"));
    assert!(header.contains("#define VERSION_MINOR 3"));
    assert!(header.contains("#define VERSION_PATCH 4"));
}

#[test]
fn test_resolve_path() {
    use crate::resolve_path;
    
    // Test relative path resolution
    let result = resolve_path("test_file");
    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.is_absolute());
    assert!(resolved.ends_with("test_file"));
    
    // Test absolute path (should remain unchanged)
    let current_dir = std::env::current_dir().unwrap();
    let absolute_path = current_dir.join("absolute_test");
    let result = resolve_path(absolute_path.to_str().unwrap());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), absolute_path);
}

#[test]
fn test_ensure_directory_exists() {
    use crate::ensure_directory_exists;
    
    let test_dir = "test_dir_structure/nested/deep";
    let test_file = format!("{}/version", test_dir);
    let test_path = Path::new(&test_file);
    
    // Ensure directory doesn't exist initially
    if test_path.parent().unwrap().exists() {
        fs::remove_dir_all(test_path.parent().unwrap()).unwrap();
    }
    
    // Test directory creation
    let result = ensure_directory_exists(test_path);
    assert!(result.is_ok());
    assert!(test_path.parent().unwrap().exists());
    
    // Clean up
    fs::remove_dir_all(test_path.parent().unwrap()).unwrap();
}

#[test]
fn test_integration_directory_paths() {
    let test_dir = "test_project/subdir";
    let version_file = format!("{}/version", test_dir);
    
    // Clean up any existing test files
    if Path::new(&test_dir).exists() {
        fs::remove_dir_all(test_dir).unwrap();
    }
    
    let bin_path = get_binary_path();

    // Test creating version file in a new directory
    let output = Command::new(&bin_path)
        .args([&version_file, "--patch"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Creating a new one"));

    // Verify directory and file were created
    assert!(Path::new(&test_dir).exists());
    assert!(Path::new(&version_file).exists());
    
    let content = fs::read_to_string(&version_file).unwrap();
    assert!(content.contains("MAJOR=0"));
    assert!(content.contains("MINOR=1"));
    assert!(content.contains("PATCH=1")); // Default 0.1.0 + patch bump = 0.1.1

    // Test bumping in the directory
    let output = Command::new(&bin_path)
        .args([&version_file, "--minor"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Version bumped to 0.2.0"));

    // Clean up
    fs::remove_dir_all(test_dir).unwrap();
}

#[test]
fn test_integration_nonexistent_file() {
    let version_file = "test_nonexistent_version";

    if Path::new(version_file).exists() {
        fs::remove_file(version_file).unwrap();
    }

    let bin_path = get_binary_path();

    let output = Command::new(&bin_path)
        .args([version_file, "--patch"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Creating a new one"));

    assert!(Path::new(version_file).exists());
    let content = fs::read_to_string(version_file).unwrap();
    assert!(content.contains("MAJOR=0"));
    assert!(content.contains("MINOR=1"));
    assert!(content.contains("PATCH=1")); // Default is 0.1.0, patch bump makes it 0.1.1

    fs::remove_file(version_file).unwrap();
}

#[test]
fn test_integration_no_flags() {
    let version_file = "test_no_flags_version";
    fs::write(version_file, "MAJOR=1\nMINOR=2\nPATCH=3\n").unwrap();

    let bin_path = get_binary_path();

    let output = Command::new(&bin_path)
        .args([version_file])
        .output()
        .expect("Failed to execute command");

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

//write a test_integration for multiple version files in different directories
#[test]
fn test_integration_multiple_version_files() {
    let version_file = "test_multiple_version_files/version";
    let version_file2 = "test_multiple_version_files/version2";

    let bin_path = get_binary_path();

    let output = Command::new(&bin_path)
        .args([version_file, "--patch"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Creating a new one"));

    let output = Command::new(&bin_path)
        .args([version_file2, "--patch"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Creating a new one"));

    assert!(Path::new(version_file).exists());
    assert!(Path::new(version_file2).exists());

    let content = fs::read_to_string(version_file).unwrap();
    assert!(content.contains("MAJOR=0"));
    assert!(content.contains("MINOR=1"));
    assert!(content.contains("PATCH=1")); // Default is 0.1.0, patch bump makes it 0.1.1

    let content = fs::read_to_string(version_file2).unwrap();
    assert!(content.contains("MAJOR=0"));
    assert!(content.contains("MINOR=1"));
    assert!(content.contains("PATCH=1")); // Default is 0.1.0, patch bump makes it 0.1.1

    fs::remove_dir_all("test_multiple_version_files").unwrap();
}
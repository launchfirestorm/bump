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
        version_type: VersionType::CalVer { suffix: 0 },
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

    // Check for [calver] section
    assert!(content.contains("[calver]"));
    assert!(content.contains("prefix = \"\""));
    assert!(content.contains("format = \"%Y.%m.%d\""));

    // Check for conflict section with comments
    assert!(content.contains("[calver.conflict]"));
    assert!(content.contains("resolution = \"suffix\""));
    assert!(content.contains("suffix = 0"));
    assert!(content.contains("delimiter = \"-\""));
    assert!(content.contains("Conflict resolution"));

    // Verify it can be read back
    let read_version = Version::from_file(&config_path).unwrap();
    match &read_version.version_type {
        VersionType::CalVer { suffix } => {
            assert_eq!(*suffix, 0);
        },
        _ => panic!("Expected CalVer version type"),
    }
    assert_eq!(read_version.prefix, "");
}


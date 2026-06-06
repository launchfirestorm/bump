use super::*;
use crate::bump::meta;
use crate::cli;

fn meta_matches(bumpfile: &str, extra: &[&str]) -> clap::ArgMatches {
    let mut args = vec!["bump"];
    args.extend_from_slice(extra);
    args.push(bumpfile);
    cli::cli().get_matches_from(args)
}

#[test]
fn meta_updates_prefix_in_bumpfile() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 2, 3, 0));

    with_cwd(temp_dir.path(), || {
        meta(&meta_matches("bump.toml", &["--prefix", "release-"])).unwrap();
    });

    let version = Version::from_file(&bump_path).unwrap();
    assert_eq!(version.version.prefix, "release-");
}

#[test]
fn meta_updates_prefix_to_empty_string() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 2, 3, 0));

    with_cwd(temp_dir.path(), || {
        meta(&meta_matches("bump.toml", &["--prefix", ""])).unwrap();
    });

    let version = Version::from_file(&bump_path).unwrap();
    assert_eq!(version.version.prefix, "");
}

#[test]
fn meta_updates_suffix_mode_to_branch() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 2, 3, 0));

    with_cwd(temp_dir.path(), || {
        meta(&meta_matches("bump.toml", &["--suffix", "branch"])).unwrap();
    });

    let version = Version::from_file(&bump_path).unwrap();
    assert_eq!(version.suffix.mode, "branch");
}

#[test]
fn meta_updates_suffix_mode_to_git_sha() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 2, 3, 0));

    with_cwd(temp_dir.path(), || {
        meta(&meta_matches("bump.toml", &["--suffix", "branch"])).unwrap();
        meta(&meta_matches("bump.toml", &["--suffix", "git_sha"])).unwrap();
    });

    let version = Version::from_file(&bump_path).unwrap();
    assert_eq!(version.suffix.mode, "git_sha");
}

#[test]
fn meta_rejects_invalid_suffix_mode() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 2, 3, 0));

    let err = with_cwd(temp_dir.path(), || {
        meta(&meta_matches("bump.toml", &["--suffix", "timestamp"]))
    })
    .unwrap_err();

    match err {
        BumpError::LogicError(msg) => {
            assert!(msg.contains("Invalid suffix mode"));
            assert!(msg.contains("timestamp"));
        }
        _ => panic!("expected LogicError"),
    }

    let version = Version::from_file(&bump_path).unwrap();
    assert_eq!(version.suffix.mode, "git_sha");
}

#[test]
fn meta_applies_prefix_then_suffix_in_separate_invocations() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (1, 2, 3, 0));

    with_cwd(temp_dir.path(), || {
        meta(&meta_matches("bump.toml", &["--prefix", "pkg-"])).unwrap();
        meta(&meta_matches("bump.toml", &["--suffix", "branch"])).unwrap();
    });

    let version = Version::from_file(&bump_path).unwrap();
    assert_eq!(version.version.prefix, "pkg-");
    assert_eq!(version.suffix.mode, "branch");
}

#[test]
fn meta_prefix_affects_printed_version() {
    let temp_dir = TempDir::new().unwrap();
    let bump_path = temp_dir.path().join("bump.toml");
    write_test_config(&bump_path, (2, 0, 1, 0));

    with_cwd(temp_dir.path(), || {
        meta(&meta_matches("bump.toml", &["--prefix", "rel-"])).unwrap();
    });

    let version = Version::from_file(&bump_path).unwrap();
    assert_eq!(
        version.to_string(&PrintType::Regular).unwrap(),
        "rel-2.0.1-rc"
    );
}

#[test]
fn meta_suffix_branch_appears_in_printed_version() {
    let repo = create_temp_git_repo(false);
    let bump_path = repo.path().join("bump.toml");
    write_bump_toml(
        &bump_path,
        r#"[timestamp]
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
mode = "git_sha"
delimiter = "+"
"#,
    );

    with_cwd(repo.path(), || {
        meta(&meta_matches("bump.toml", &["--suffix", "branch"])).unwrap();
    });

    let version = Version::from_file(&bump_path).unwrap();
    let branch = run_git_in_output(repo.path(), &["branch", "--show-current"]);
    assert_eq!(
        version.to_string(&PrintType::WithSuffix).unwrap(),
        format!("v1.0.0+{branch}")
    );
}

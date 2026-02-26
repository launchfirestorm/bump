// Test utilities and shared helpers
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tempfile::TempDir;

// Import types from bump module
pub use crate::bump::{
    BumpError, BumpType, PointType, ensure_directory_exists, get_git_branch,
    get_git_commit_sha, resolve_path,
};

// Import types from version module
pub use crate::version::{
    CandidateSection, Config, Config as BumpConfig, DevelopmentSection, Version,
    SemVerConfig, SemVerFormatSection, SemVerVersionSection,
    CalVerConfig, CalVerConflictSection, CalVerFormatSection, CalVerVersionSection, VersionType,
};

// RAII wrapper for test directories that automatically sets thread-local repo path
pub struct TestRepo {
    _temp_dir: TempDir,
}

impl TestRepo {
    pub fn new(temp_dir: TempDir) -> Self {
        crate::bump::set_test_repo_path(Some(temp_dir.path().to_path_buf()));
        TestRepo { _temp_dir: temp_dir }
    }

    pub fn path(&self) -> &Path {
        self._temp_dir.path()
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        crate::bump::set_test_repo_path(None);
    }
}

static TEST_GIT_CONFIG: OnceLock<PathBuf> = OnceLock::new();

pub fn get_test_git_config() -> &'static Path {
    TEST_GIT_CONFIG.get_or_init(|| {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-gitconfig");
        let config_content = r#"[user]
	email = test@example.com
	name = Test User
[commit]
	gpgsign = false
[tag]
	gpgsign = false
"#;
        fs::write(&config_path, config_content).unwrap();
        // Leak the TempDir so it persists for the entire test run
        let path = config_path.clone();
        std::mem::forget(temp_dir);
        path
    })
}

pub fn run_git_in(path: &Path, args: &[&str]) {
    let output = Command::new("git")
        .env("GIT_CONFIG_GLOBAL", get_test_git_config())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run git -C {} {:?}: {}", path.display(), args, err));

    if !output.status.success() {
        panic!(
            "git -C {} {:?} failed: {}",
            path.display(),
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

pub fn run_git_in_output(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .env("GIT_CONFIG_GLOBAL", get_test_git_config())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to run git -C {} {:?}: {}", path.display(), args, err));

    if !output.status.success() {
        panic!(
            "git -C {} {:?} failed: {}",
            path.display(),
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn init_repo(path: &Path) {
    run_git_in(path, &["init"]);
    run_git_in(path, &["commit", "--allow-empty", "-m", "Initial commit"]);
}

pub fn create_temp_git_repo(tagged: bool) -> TestRepo {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    init_repo(repo_path);
    if tagged {
        run_git_in(repo_path, &["tag", "v1.2.3"]);
    }
    TestRepo::new(temp_dir)
}

pub fn create_temp_dir() -> TestRepo {
    let temp_dir = TempDir::new().unwrap();
    TestRepo::new(temp_dir)
}

pub fn git_rev_parse_short_in(path: &Path) -> String {
    run_git_in_output(path, &["rev-parse", "--short", "HEAD"])
}

pub fn write_bump_toml(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
}

pub fn write_test_config(path: &Path, version: (u32, u32, u32, u32)) {
    let (major, minor, patch, candidate) = version;
    let content = format!(r#"[semver.format]
prefix = "v"
delimiter = "."

[semver.version]
major = {}
minor = {}
patch = {}
candidate = {}

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#, major, minor, patch, candidate);
    fs::write(path, content).unwrap();
}

pub fn write_test_config_with_timestamp(path: &Path, version: (u32, u32, u32, u32), timestamp_format: &str) {
    let (major, minor, patch, candidate) = version;
    let content = format!(r#"[semver.format]
prefix = "v"
delimiter = "."
timestamp = "{}"

[semver.version]
major = {}
minor = {}
patch = {}
candidate = {}

[semver.candidate]
promotion = "minor"
delimiter = "-rc"

[semver.development]
promotion = "git_sha"
delimiter = "+"
"#, timestamp_format, major, minor, patch, candidate);
    fs::write(path, content).unwrap();
}

pub fn make_default_config(major: u32, minor: u32, patch: u32, candidate: u32) -> BumpConfig {
    BumpConfig::SemVer(SemVerConfig {
        format: SemVerFormatSection {
            prefix: "v".to_string(),
            delimiter: ".".to_string(),
            timestamp: None,
        },
        version: SemVerVersionSection {
            major,
            minor,
            patch,
            candidate,
        },
        candidate: CandidateSection {
            promotion: "minor".to_string(),
            delimiter: "-rc".to_string(),
        },
        development: DevelopmentSection {
            promotion: "git_sha".to_string(),
            delimiter: "+".to_string(),
        },
    })
}

#[allow(dead_code)]
pub fn write_calver_config(path: &Path, suffix: u32) {
    let content = format!(r#"[calver]
prefix = ""
format = "%Y.%m.%d"

[calver.conflict]
resolution = "suffix"
suffix = {}
delimiter = "-"
"#, suffix);
    fs::write(path, content).unwrap();
}

#[allow(dead_code)]
pub fn make_calver_config(revision: u32) -> BumpConfig {
    let now = chrono::Utc::now();
    BumpConfig::CalVer(CalVerConfig {
        format: CalVerFormatSection {
            prefix: "".to_string(),
            delimiter: ".".to_string(),
            year: "%Y".to_string(),
            month: Some("%m".to_string()),
            day: Some("%d".to_string()),
        },
        version: CalVerVersionSection {
            year: now.format("%Y").to_string(),
            month: Some(now.format("%m").to_string()),
            day: Some(now.format("%d").to_string()),
        },
        conflict: CalVerConflictSection {
            revision,
            delimiter: "-".to_string(),
        },
    })
}

// Test modules
mod semver;
mod calver;

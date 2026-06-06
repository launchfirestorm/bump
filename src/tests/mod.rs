use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

pub use crate::bump::{BumpError, BumpType, PrintType, create_git_tag};
pub use crate::version::{PhaseTable, SuffixTable, TimestampTable, Version, VersionTable};

pub struct TestRepo {
    temp_dir: TempDir,
}

impl TestRepo {
    pub fn new(temp_dir: TempDir) -> Self {
        crate::bump::set_test_repo_path(Some(temp_dir.path().to_path_buf()));
        Self { temp_dir }
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        crate::bump::set_test_repo_path(None);
    }
}

static TEST_GIT_CONFIG: OnceLock<PathBuf> = OnceLock::new();
static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn get_test_git_config() -> &'static Path {
    TEST_GIT_CONFIG.get_or_init(|| {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-gitconfig");
        let config_content = r"[user]
	email = test@example.com
	name = Test User
[commit]
	gpgsign = false
[tag]
	gpgsign = false
";
        fs::write(&config_path, config_content).unwrap();
        let leaked = config_path;
        std::mem::forget(temp_dir);
        leaked
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
        .unwrap_or_else(|err| {
            panic!(
                "failed to run git -C {} {:?}: {}",
                path.display(),
                args,
                err
            )
        });

    assert!(
        output.status.success(),
        "git -C {} {:?} failed: {}",
        path.display(),
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn run_git_in_output(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .env("GIT_CONFIG_GLOBAL", get_test_git_config())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "failed to run git -C {} {:?}: {}",
                path.display(),
                args,
                err
            )
        });

    assert!(
        output.status.success(),
        "git -C {} {:?} failed: {}",
        path.display(),
        args,
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn init_repo(path: &Path) {
    run_git_in(path, &["init"]);
    run_git_in(path, &["config", "user.name", "Test User"]);
    run_git_in(path, &["config", "user.email", "test@example.com"]);
    run_git_in(path, &["config", "commit.gpgsign", "false"]);
    run_git_in(path, &["config", "tag.gpgsign", "false"]);
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
    let (major, minor, patch, distance) = version;
    let content = format!(
        r#"[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"
last = "2026-01-01 00:00:00 UTC"

[version]
mode = "semver"
prefix = "v"
delimiter = "."
major = {major}
minor = {minor}
patch = {patch}

[phase]
prefix = "-"
name = "rc"
delimiter = "-"
distance = {distance}

[suffix]
mode = "git_sha"
delimiter = "+"
"#
    );
    fs::write(path, content).unwrap();
}

pub fn make_semver(prefix: &str, major: u32, minor: u32, patch: u32, candidate: u32) -> Version {
    Version {
        path: PathBuf::from("test.toml"),
        timestamp: TimestampTable {
            format: "%Y-%m-%d %H:%M:%S %Z".to_string(),
            last: "2026-01-01 00:00:00 UTC".to_string(),
        },
        version: VersionTable {
            mode: "semver".to_string(),
            prefix: prefix.to_string(),
            delimiter: ".".to_string(),
            major,
            minor: Some(minor),
            patch: Some(patch),
        },
        phase: PhaseTable {
            prefix: "-".to_string(),
            name: if candidate > 0 {
                "rc".to_string()
            } else {
                String::new()
            },
            delimiter: "-".to_string(),
            distance: candidate,
        },
        suffix: SuffixTable {
            mode: "git_sha".to_string(),
            delimiter: "+".to_string(),
        },
    }
}

pub fn make_calver(prefix: &str) -> Version {
    Version {
        path: PathBuf::from("test.toml"),
        timestamp: TimestampTable {
            format: "%Y-%m-%d %H:%M:%S %Z".to_string(),
            last: "2026-01-01 00:00:00 UTC".to_string(),
        },
        version: VersionTable {
            mode: "calver".to_string(),
            prefix: prefix.to_string(),
            delimiter: ".".to_string(),
            major: 2026,
            minor: Some(6),
            patch: Some(5),
        },
        phase: PhaseTable {
            prefix: "-".to_string(),
            name: String::new(),
            delimiter: "-".to_string(),
            distance: 0,
        },
        suffix: SuffixTable {
            mode: "git_sha".to_string(),
            delimiter: "+".to_string(),
        },
    }
}

pub fn with_cwd<T>(dir: &Path, f: impl FnOnce() -> T) -> T {
    let lock = CWD_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().unwrap();
    let previous = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir).unwrap();
    let output = f();
    std::env::set_current_dir(previous).unwrap();
    output
}

mod calver;
mod codegen;
mod meta;
mod semver;

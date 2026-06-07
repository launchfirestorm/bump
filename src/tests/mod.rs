use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

pub use crate::bump::{BumpError, BumpType, create_git_tag};
pub use crate::print::PrintType;
pub use crate::version::{
    Base, Label, LabelPosition, Phase, Suffix, SuffixMode, Timestamp, Version, VersionMode,
};

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

pub const TEST_TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S %Z";

pub fn test_timestamp_last() -> String {
    chrono::Utc::now()
        .format(TEST_TIMESTAMP_FORMAT)
        .to_string()
}

pub fn test_timestamp() -> Timestamp {
    Timestamp {
        format: TEST_TIMESTAMP_FORMAT.to_string(),
        last: test_timestamp_last(),
    }
}

pub fn default_label() -> Label {
    Label {
        position: LabelPosition::AfterBase,
    }
}

pub fn write_bump_toml(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
}

pub fn write_semver_config(path: &Path, version: (u32, u32, u32, u32)) {
    let (major, minor, patch, distance) = version;
    let phase_name = if distance > 0 { "rc" } else { "" };
    let content = format!(
        r#"prefix = "v"

[base]
mode = "semver"
delimiter = "."
major = {major}
minor = {minor}
patch = {patch}

[phase]
separator = "-"
name = "{phase_name}"
delimiter = "."
distance = {distance}

[suffix]
mode = "git_sha"
separator = "+"

[timestamp]
format = "{TEST_TIMESTAMP_FORMAT}"
last = "{last}"

[label]
position = "after-base"
"#,
        last = test_timestamp_last(),
    );
    fs::write(path, content).unwrap();
}

pub fn write_calver_config(path: &Path, version: (u32, u32, u32, u32)) {
    let (year, month, day, distance) = version;
    let content = format!(
        r#"prefix = ""

[base]
mode = "calver"
delimiter = "."
major = {year}
minor = {month}
patch = {day}

[phase]
separator = "-"
name = ""
delimiter = "."
distance = {distance}

[suffix]
mode = "git_sha"
separator = "+"

[timestamp]
format = "{TEST_TIMESTAMP_FORMAT}"
last = "{last}"

[label]
position = "after-base"
"#,
        last = test_timestamp_last(),
    );
    fs::write(path, content).unwrap();
}

pub fn make_semver(prefix: &str, major: u32, minor: u32, patch: u32, candidate: u32) -> Version {
    Version {
        path: PathBuf::from("test.toml"),
        timestamp: test_timestamp(),
        prefix: prefix.to_string(),
        base: Base {
            mode: VersionMode::Semver,
            delimiter: ".".to_string(),
            major,
            minor: Some(minor),
            patch: Some(patch),
        },
        phase: Phase {
            separator: "-".to_string(),
            name: if candidate > 0 {
                "rc".to_string()
            } else {
                String::new()
            },
            delimiter: "-".to_string(),
            distance: candidate,
        },
        suffix: Suffix {
            mode: SuffixMode::GitSha,
            separator: "+".to_string(),
        },
        label: default_label(),
    }
}

pub fn make_calver(prefix: &str) -> Version {
    Version {
        path: PathBuf::from("test.toml"),
        timestamp: test_timestamp(),
        prefix: prefix.to_string(),
        base: Base {
            mode: VersionMode::Calver,
            delimiter: ".".to_string(),
            major: 2026,
            minor: Some(6),
            patch: Some(5),
        },
        phase: Phase {
            separator: "-".to_string(),
            name: String::new(),
            delimiter: "-".to_string(),
            distance: 0,
        },
        suffix: Suffix {
            mode: SuffixMode::GitSha,
            separator: "+".to_string(),
        },
        label: default_label(),
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
mod print;
mod semver;

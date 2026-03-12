use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tempfile::TempDir;

pub use crate::bump::{
    BumpError, BumpType, PointType,
};
pub use crate::version::{
    default_calver, default_semver, CalVer, SemVer, Version, VersionType,
};

pub struct TestRepo {
    _temp_dir: TempDir,
}

impl TestRepo {
    pub fn new(temp_dir: TempDir) -> Self {
        crate::bump::set_test_repo_path(Some(temp_dir.path().to_path_buf()));
        Self {
            _temp_dir: temp_dir,
        }
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

fn get_test_git_config() -> &'static Path {
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
        let leaked = config_path.clone();
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
        .unwrap_or_else(|err| {
            panic!(
                "failed to run git -C {} {:?}: {}",
                path.display(),
                args,
                err
            )
        });

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
    let content = format!(
        r#"[semver.format]
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
"#,
        major, minor, patch, candidate
    );
    fs::write(path, content).unwrap();
}

pub fn write_calver_config(path: &Path, year: &str, month: Option<&str>, day: Option<&str>, revision: u32) {
    let month_fmt = month
        .map(|m| format!("month = \"{}\"\n", m))
        .unwrap_or_default();
    let day_fmt = day
        .map(|d| format!("day = \"{}\"\n", d))
        .unwrap_or_default();

    let content = format!(
        r#"[calver.format]
prefix = ""
delimiter = "."
year = "%Y"
{}{}
[calver.version]
year = "{}"
{}{}
[calver.conflict]
revision = {}
delimiter = "-"
"#,
        if month.is_some() { "month = \"%m\"\n" } else { "" },
        if day.is_some() { "day = \"%d\"\n" } else { "" },
        year,
        month_fmt,
        day_fmt,
        revision,
    );

    fs::write(path, content).unwrap();
}

pub fn make_semver(prefix: &str, major: u32, minor: u32, patch: u32, candidate: u32) -> Version {
    Version {
        version_type: VersionType::SemVer(default_semver(prefix, major, minor, patch, candidate)),
        path: PathBuf::from("test.toml"),
    }
}

pub fn make_calver(prefix: &str) -> Version {
    Version {
        version_type: VersionType::CalVer(default_calver(prefix)),
        path: PathBuf::from("test.toml"),
    }
}

mod semver;
mod calver;

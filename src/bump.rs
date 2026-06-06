use crate::lang::{self, Language};
use crate::version::Version;
use clap::ArgMatches;
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
};

#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
thread_local! {
    /// Test-only: allows tests to override the git repository path without changing CWD
    static TEST_REPO_PATH: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[cfg(test)]
/// Test-only: Set the repository path for git operations in this thread
pub fn set_test_repo_path(path: Option<PathBuf>) {
    TEST_REPO_PATH.with(|p| *p.borrow_mut() = path);
}

pub enum BumpType {
    Major,
    Minor,
    Patch,
    Phase(String), // increment phase distance
    Calendar,
}

#[derive(Debug)]
pub enum BumpError {
    IoError(io::Error),
    ParseError(String),
    LogicError(String),
    Git(String),
}

pub enum PrintType {
    OnlyPrefix,
    OnlyPhase,
    OnlyBase,
    Regular,
    NoPrefix,
    NoPhase,
    WithSuffix,
    WithTimestamp,
    Full, // includes prefix, phase, suffix, and timestamp
}

impl fmt::Display for BumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    write!(f, "bump error: I/O >> file not found '{err}'")
                } else {
                    write!(f, "bump error: I/O >> {err}")
                }
            }
            Self::ParseError(field) => write!(f, "bump error: parse >> {field}"),
            Self::LogicError(msg) => write!(f, "bump error >> {msg}"),
            Self::Git(msg) => write!(f, "bump error: git >> {msg}"),
        }
    }
}

impl From<io::Error> for BumpError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

pub fn resolve_path(input_path: &str) -> PathBuf {
    let path = Path::new(input_path);

    if path.is_absolute() {
        // Absolute path - return as is
        path.to_path_buf()
    } else {
        // Relative path - resolve relative to current directory
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

pub fn ensure_directory_exists(path: &Path) -> Result<(), BumpError> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(BumpError::IoError)?;
    }
    Ok(())
}

pub fn has_meta_flags(matches: &ArgMatches) -> bool {
    matches.get_one::<String>("prefix").is_some() || matches.get_one::<String>("suffix").is_some()
}

pub fn get_version(matches: &ArgMatches) -> Result<Version, BumpError> {
    let version_file_path = matches
        .get_one::<String>("bumpfile")
        .expect("PATH not provided");
    let version_path = resolve_path(version_file_path);
    Version::from_file(&version_path)
}

pub fn get_bump_type(matches: &ArgMatches) -> Result<BumpType, BumpError> {
    if matches.get_flag("major") {
        Ok(BumpType::Major)
    } else if matches.get_flag("minor") {
        Ok(BumpType::Minor)
    } else if matches.get_flag("patch") {
        Ok(BumpType::Patch)
    } else if let Some(phase_value) = matches.get_one::<String>("phase") {
        Ok(BumpType::Phase(phase_value.clone()))
    } else if matches.get_flag("calendar") {
        Ok(BumpType::Calendar)
    } else {
        Err(BumpError::LogicError(
            "No valid bump type specified".to_string(),
        ))
    }
}

pub fn get_print_type(matches: &ArgMatches) -> PrintType {
    if matches.get_flag("only-prefix") {
        PrintType::OnlyPrefix
    } else if matches.get_flag("only-phase") {
        PrintType::OnlyPhase
    } else if matches.get_flag("only-base") {
        PrintType::OnlyBase
    } else if matches.get_flag("no-prefix") {
        PrintType::NoPrefix
    } else if matches.get_flag("no-phase") {
        PrintType::NoPhase
    } else if matches.get_flag("with-suffix") {
        PrintType::WithSuffix
    } else if matches.get_flag("with-timestamp") {
        PrintType::WithTimestamp
    } else if matches.get_flag("full") {
        PrintType::Full
    } else {
        PrintType::Regular
    }
}

pub fn initialize(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = matches.get_one::<String>("bumpfile").unwrap();
    let filepath = resolve_path(bumpfile);
    ensure_directory_exists(&filepath)?;
    let version = Version::default(&filepath);
    version.create_file()?;
    println!("Initialized new BUMPFILE at '{}'", filepath.display());
    Ok(())
}

pub fn print(matches: &ArgMatches) -> Result<(), BumpError> {
    let print_type = get_print_type(matches);
    let bumpfile = matches.get_one::<String>("bumpfile").unwrap();
    let version = Version::from_file(&resolve_path(bumpfile))?;
    print!("{}", version.to_string(&print_type)?);
    Ok(())
}

pub fn apply(matches: &ArgMatches) -> Result<(), BumpError> {
    let mut version = get_version(matches)?;
    let has_meta = has_meta_flags(matches);
    let has_formal = matches.contains_id("formal");

    if let Some(prefix) = matches.get_one::<String>("prefix") {
        version.version.prefix.clone_from(prefix);
    }
    if let Some(suffix) = matches.get_one::<String>("suffix") {
        if suffix != "git_sha" && suffix != "branch" {
            return Err(BumpError::LogicError(format!(
                "Invalid suffix mode: '{suffix}'. Expected 'git_sha' or 'branch'."
            )));
        }
        version.suffix.mode.clone_from(suffix);
    }

    if has_formal {
        version.bump(&get_bump_type(matches)?)?;
        println!(
            "bumped {} to {}",
            version.path.display(),
            version.to_string(&PrintType::WithTimestamp)?
        );
    }

    if has_meta || has_formal {
        version.to_file()?;
    }

    Ok(())
}

fn git_cmd() -> ProcessCommand {
    #[cfg(test)]
    {
        let mut cmd = ProcessCommand::new("git");
        TEST_REPO_PATH.with(|p| {
            if let Some(ref path) = *p.borrow() {
                cmd.arg("-C").arg(path);
            }
        });
        return cmd;
    }

    #[cfg(not(test))]
    ProcessCommand::new("git")
}

pub fn run_git(command: &str) -> Result<String, BumpError> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let output = git_cmd()
        .args(&args)
        .output()
        .map_err(|e| BumpError::Git(format!("git {command}: {e}")))?;
    if !output.status.success() {
        return Err(BumpError::Git(format!(
            "git {command}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return Ok(stdout);
    }
    Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

pub fn is_git_repository() -> bool {
    git_cmd()
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_git_commit_sha() -> Result<String, BumpError> {
    run_git("rev-parse --short HEAD")
}

pub fn get_git_branch() -> Result<String, BumpError> {
    run_git("rev-parse --abbrev-ref HEAD")
}

pub fn generate(matches: &ArgMatches, lang: Language) -> Result<(), BumpError> {
    let bumpfile = matches.get_one::<String>("bumpfile").unwrap();
    let version = Version::from_file(&resolve_path(bumpfile))?;
    let output_files: Vec<&String> = matches.get_many::<String>("output").unwrap().collect();
    for output_file in output_files {
        let output_path = Path::new(output_file);

        ensure_directory_exists(output_path)?;
        lang::output_file(lang, &version, output_path)?;
    }

    Ok(())
}

fn git_tag_exists(tag_name: &str) -> Result<bool, BumpError> {
    let output = git_cmd()
        .args([
            "rev-parse",
            "-q",
            "--verify",
            &format!("refs/tags/{tag_name}"),
        ])
        .output()
        .map_err(|e| BumpError::Git(format!("failed to check if tag exists: {e}")))?;

    Ok(output.status.success())
}

pub fn create_git_tag(version: &Version, message: Option<&str>) -> Result<(), BumpError> {
    if !is_git_repository() {
        return Err(BumpError::LogicError("Not in a git repository".to_string()));
    }

    let tag_name = version.to_string(&PrintType::Regular)?;

    if git_tag_exists(&tag_name)? {
        return Err(BumpError::Git(format!("Tag '{tag_name}' already exists")));
    }

    let mut cmd = git_cmd();
    cmd.args(["tag", "-a", &tag_name]);

    if let Some(msg) = message {
        cmd.args(["-m", msg]);
    } else {
        let default_message = format!("chore(release): bump version to {tag_name}");
        cmd.args(["-m", &default_message]);
    }

    let output = cmd
        .output()
        .map_err(|e| BumpError::Git(format!("failed to create git tag: {e}")))?;

    if !output.status.success() {
        return Err(BumpError::Git(format!(
            "failed to create tag '{tag_name}': {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    println!("Created git tag: {tag_name}");
    Ok(())
}

pub fn tag_version(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = matches.get_one::<String>("bumpfile").unwrap();
    let version = Version::from_file(&resolve_path(bumpfile))?;
    let message = matches.get_one::<String>("message");

    create_git_tag(&version, message.map(String::as_str))
}

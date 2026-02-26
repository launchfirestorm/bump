use crate::lang::{self, Language};
use crate::version::{default_semver_config, Version, VersionType, Config};
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

pub enum PointType {
    Major,
    Minor,
    Patch,
}

pub enum BumpType {
    Prefix(String),
    Point(PointType),
    Candidate, // candidate will bump the minor version and append a rc1
    Release,   // release will drop candidacy and not increment (hence released)
    Base,
}

#[derive(Debug)]
pub enum BumpError {
    IoError(io::Error),
    ParseError(String),
    TomlError(toml::de::Error),
    LogicError(String),
    Git(String),
}

impl fmt::Display for BumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BumpError::IoError(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    write!(f, "bump error: I/O >> file not found '{}'", err)
                } else {
                    write!(f, "bump error: I/O >> {err}")
                }
            }
            BumpError::ParseError(field) => write!(f, "bump error: parse >> {field}"),
            BumpError::TomlError(err) => write!(f, "bump error: config >> {err}"),
            BumpError::LogicError(msg) => write!(f, "bump error >> {msg}"),
            BumpError::Git(msg) => write!(f, "bump error: git >> {msg}"),
        }
    }
}

impl From<io::Error> for BumpError {
    fn from(err: io::Error) -> Self {
        BumpError::IoError(err)
    }
}

impl From<toml::de::Error> for BumpError {
    fn from(err: toml::de::Error) -> Self {
        BumpError::TomlError(err)
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

pub fn prompt_for_version(path: &Path) -> Result<Version, BumpError> {
    let mut version_input = String::new();
    println!("Enter the version number (e.g. 1.2.3) or press enter for default (0.1.0): ");

    io::stdin()
        .read_line(&mut version_input)
        .map_err(BumpError::IoError)?;
    let version_input = version_input.trim();

    if version_input.is_empty() {
        Ok(Version::default(path))
    } else {
        let version_parts: Result<Vec<u32>, _> =
            version_input.split('.').map(|s| s.parse::<u32>()).collect();

        match version_parts {
            Ok(parts) if parts.len() == 3 => {
                let config = default_semver_config("v".to_string(), parts[0], parts[1], parts[2], 0);

                Ok(Version {
                    prefix: "v".to_string(),
                    timestamp: None,
                    version_type: VersionType::SemVer {
                        major: parts[0],
                        minor: parts[1],
                        patch: parts[2],
                        candidate: 0,
                    },
                    path: path.to_path_buf(),
                    config,
                })
            }
            _ => Err(BumpError::ParseError("invalid version format".to_string())),
        }
    }
}

pub fn get_version(matches: &ArgMatches) -> Result<Version, BumpError> {
    let version_file_path = matches
        .get_one::<String>("bumpfile")
        .expect("PATH not provided");
    let version_path = resolve_path(version_file_path);
    Version::from_file(&version_path)
}

pub fn get_bump_type(matches: &ArgMatches) -> Result<BumpType, BumpError> {
    if matches.get_one::<String>("prefix").is_some() {
        Ok(BumpType::Prefix(
            matches.get_one::<String>("prefix").unwrap().to_string(),
        ))
    } else if matches.get_flag("major") {
        Ok(BumpType::Point(PointType::Major))
    } else if matches.get_flag("minor") {
        Ok(BumpType::Point(PointType::Minor))
    } else if matches.get_flag("patch") {
        Ok(BumpType::Point(PointType::Patch))
    } else if matches.get_flag("candidate") {
        Ok(BumpType::Candidate)
    } else if matches.get_flag("release") {
        Ok(BumpType::Release)
    } else {
        Err(BumpError::LogicError(
            "No valid bump type specified".to_string(),
        ))
    }
}

pub fn initialize(bumpfile: &str, prefix: &str, use_calver: bool) -> Result<(), BumpError> {
    let filepath = resolve_path(bumpfile);
    ensure_directory_exists(&filepath)?;

    if use_calver {
        // CalVer - no git tag detection, just create config
        let version = Version {
            prefix: prefix.to_string(),
            timestamp: None,
            version_type: VersionType::CalVer { suffix: 0 },
            path: filepath.clone(),
            config: crate::version::default_calver_config(prefix.to_string()),
        };
        version.file_init()?;
        println!("Initialized new CalVer version file at '{}'", filepath.display());
    } else {
        // SemVer - prompt for tag or manual
        let mut use_git_tag = String::new();
        println!("Use git tag for versioning? (y/n): ");
        io::stdin()
            .read_line(&mut use_git_tag)
            .map_err(BumpError::IoError)?;
        let use_git_tag = use_git_tag.trim().to_lowercase();

        if use_git_tag == "y" {
            match get_git_tag(true) {
                Ok(git_tag) => {
                    println!("Found git tag: {git_tag}");
                    let mut git_version = Version::from_string(&git_tag, &filepath)?;
                    git_version.prefix = prefix.to_string(); // Override prefix from CLI
                    git_version.file_init()?;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        } else {
            let mut version = prompt_for_version(&filepath)?;
            version.prefix = prefix.to_string(); // Override prefix from CLI
            version.file_init()?;
        }

        println!("Initialized new SemVer version file at '{}'", filepath.display());
    }

    Ok(())
}

pub fn print(version: &Version, base: bool) {
    let bump_type = if base {
        BumpType::Base
    } else {
        match &version.version_type {
            VersionType::SemVer { candidate, .. } if *candidate > 0 => BumpType::Candidate,
            _ => BumpType::Point(PointType::Patch), // bump_type doesn't matter here
        }
    };
    print!("{}", version.to_string(&bump_type));
}

pub fn print_with_timestamp(version: &Version) {
    // bump_type doesn't matter here
    let bump_type = BumpType::Point(PointType::Patch);
    if let Some(timestamp) = &version.timestamp {
        print!("{} (built on {})", version.to_string(&bump_type), timestamp);
    } else {
        print!("{}", version.to_string(&bump_type));
    }
}

pub fn apply(matches: &ArgMatches) -> Result<(), BumpError> {
    let mut version = get_version(matches)?;
    let bump_type = get_bump_type(matches)?;
    
    // Validate that bump type is compatible with version type
    match (&version.version_type, &bump_type) {
        (VersionType::CalVer { .. }, BumpType::Point(_)) => {
            return Err(BumpError::LogicError(
                "CalVer does not support major/minor/patch bumps. Use 'bump' without arguments to regenerate date-based version.".to_string()
            ));
        }
        (VersionType::CalVer { .. }, BumpType::Candidate) => {
            return Err(BumpError::LogicError(
                "CalVer does not support candidate versions. Use conflict resolution in bump.toml instead.".to_string()
            ));
        }
        (VersionType::CalVer { .. }, BumpType::Release) => {
            return Err(BumpError::LogicError(
                "CalVer does not support release bumps.".to_string()
            ));
        }
        _ => {} // Valid combination
    }
    
    version.bump(&bump_type)?;

    match version.to_file() {
        Ok(()) => match bump_type {
            BumpType::Prefix(new_prefix) => println!(
                "Updated prefix of '{}' to '{}'",
                version.path.display(),
                new_prefix
            ),
            BumpType::Point(_) => println!(
                "Bumped '{}' to point release {}",
                version.path.display(),
                version.to_string(&bump_type)
            ),
            BumpType::Candidate => println!(
                "Bumped '{}' to new candidate {}",
                version.path.display(),
                version.to_string(&bump_type)
            ),
            BumpType::Release => println!(
                "Bumped '{}' drop candidacy to release! {}",
                version.path.display(),
                version.to_string(&bump_type)
            ),
            BumpType::Base => { /* won't happen */ }
        },
        Err(err) => {
            return Err(err);
        }
    }

    Ok(())
}

pub fn run_git(command: &str) -> Result<String, BumpError> {
    let args: Vec<&str> = command.split_whitespace().collect();
    let mut cmd = ProcessCommand::new("git");
    
    #[cfg(test)]
    {
        // Check if test has set a specific repo path
        TEST_REPO_PATH.with(|p| {
            if let Some(ref path) = *p.borrow() {
                cmd.arg("-C").arg(path);
            }
        });
    }
    
    let output = cmd
        .args(&args)
        .output()
        .map_err(|e| BumpError::Git(format!("git {}: {e}", command)))?;
    if !output.status.success() {
        return Err(BumpError::Git(
            format!("git {}: {}", command, String::from_utf8_lossy(&output.stderr)),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return Ok(stdout);
    }
    Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

pub fn is_git_repository() -> bool {
    let mut cmd = ProcessCommand::new("git");
    
    #[cfg(test)]
    {
        // Check if test has set a specific repo path
        TEST_REPO_PATH.with(|p| {
            if let Some(ref path) = *p.borrow() {
                cmd.arg("-C").arg(path);
            }
        });
    }
    
    cmd.args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn get_git_tag(last_tag: bool) -> Result<String, BumpError> {
    if last_tag {
        run_git("describe --tags --abbrev=0")
    } else {
        run_git("describe --exact-match --tags HEAD")
    }
}

pub fn get_git_commit_sha() -> Result<String, BumpError> {
    run_git("rev-parse --short HEAD")
}

pub fn get_git_branch() -> Result<String, BumpError> {
    run_git("rev-parse --abbrev-ref HEAD")
}

pub fn get_development_suffix(version: &Version) -> Result<String, BumpError> {
    match &version.config {
        Config::SemVer(semver_config) => {
            match semver_config.development.promotion.as_str() {
                "git_sha" => get_git_commit_sha(),
                "branch" => get_git_branch(),
                "full" => {
                    let branch = get_git_branch()?;
                    let sha = get_git_commit_sha()?;
                    Ok(format!("{}_{}", branch, sha))
                }
                _ => get_git_commit_sha(), // default to git_sha
            }
        }
        Config::CalVer(_) => {
            // CalVer doesn't use development suffixes
            Err(BumpError::LogicError("CalVer does not support development suffixes".to_string()))
        }
    }
}

pub fn generate(matches: &ArgMatches, lang: &Language) -> Result<(), BumpError> {
    let version = Version::from_argmatches(matches)?;
    let output_files: Vec<&String> = matches.get_many::<String>("output").unwrap().collect();
    for output_file in output_files {
        let output_path = Path::new(output_file);

        // Create directory if it doesn't exist (mkdir -p behavior)
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(BumpError::IoError)?;
        }
        lang::output_file(lang, &version, output_path)?;
    }

    Ok(())
}

pub fn create_git_tag(version: &Version, message: Option<&str>) -> Result<(), BumpError> {
    if !is_git_repository() {
        return Err(BumpError::LogicError("Not in a git repository".to_string()));
    }

    // Create the conventional tag name based on version
    let tag_name = match &version.version_type {
        VersionType::SemVer { major, minor, patch, candidate } => {
            match &version.config {
                Config::SemVer(semver_config) => {
                    if *candidate > 0 {
                        format!(
                            "{}{}.{}.{}{}{}",
                            version.prefix,
                            major,
                            minor,
                            patch,
                            semver_config.candidate.delimiter,
                            candidate
                        )
                    } else {
                        format!(
                            "{}{}.{}.{}",
                            version.prefix, major, minor, patch
                        )
                    }
                }
                _ => unreachable!("SemVer version type must have SemVer config"),
            }
        }
        VersionType::CalVer { suffix } => {
            match &version.config {
                Config::CalVer(calver_config) => {
                    let now = chrono::Utc::now();
                    let date_str = now.format(&calver_config.format).to_string();
                    if *suffix > 0 {
                        format!("{}{}{}{}", version.prefix, date_str, calver_config.conflict.delimiter, suffix)
                    } else {
                        format!("{}{}", version.prefix, date_str)
                    }
                }
                _ => unreachable!("CalVer version type must have CalVer config"),
            }
        }
    };

    // Check if the tag already exists
    let tag_exists = ProcessCommand::new("git")
        .args(["tag", "-l", &tag_name])
        .output()
        .map_err(|e| BumpError::Git(format!("failed to check if tag exists: {e}")))?;

    if !String::from_utf8_lossy(&tag_exists.stdout)
        .trim()
        .is_empty()
    {
        return Err(BumpError::Git(format!("Tag '{tag_name}' already exists")));
    }

    // Create the tag
    let mut cmd = ProcessCommand::new("git");
    cmd.args(["tag", "-a", &tag_name]);

    if let Some(msg) = message {
        cmd.args(["-m", msg]);
    } else {
        // Default conventional commit message
        let default_message = format!("chore(release): bump version to {}", tag_name);
        cmd.args(["-m", &default_message]);
    }

    let output = cmd
        .output()
        .map_err(|e| BumpError::Git(format!("failed to create git tag: {e}")))?;

    if !output.status.success() {
        return Err(BumpError::Git(format!(
            "failed to create tag '{}': {}",
            tag_name,
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

    create_git_tag(&version, message.map(|s| s.as_str()))
}

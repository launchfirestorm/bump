use clap::{Arg, ArgMatches, Command};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    process::{Command as ProcessCommand, ExitCode},
};

use crate::lang::Language;

mod lang;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionSection {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub candidate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CandidateSection {
    pub promotion: String, // "minor", "major", "patch"
    pub delimiter: String, // "-rc"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DevelopmentSection {
    pub promotion: String, // "git_sha", "branch", "full"
    pub delimiter: String, // "+"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BumpConfig {
    pub prefix: String,
    pub version: VersionSection,
    pub candidate: CandidateSection,
    pub development: DevelopmentSection,
}

#[derive(Debug)]
struct Version {
    pub prefix: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub candidate: u32, // will be zero for point-release
    pub path: PathBuf,
    pub config: BumpConfig,
}

#[derive(Debug)]
enum BumpError {
    IoError(io::Error),
    ParseError(String),
    TomlError(toml::de::Error),
    LogicError(String),
    Git(String),
}

enum PointType {
    Major,
    Minor,
    Patch,
}

enum BumpType {
    Prefix(String),
    Point(PointType),
    Candidate, // candidate will bump the minor version and append a rc1
    Release,   // release will drop candidacy and not increment (hence released)
    Base,
}

impl fmt::Display for BumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BumpError::IoError(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    write!(f, "bump error: I/O >> file not found '{}'", err.to_string())
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

impl Version {
    fn default(path: &Path) -> Self {
        let config = BumpConfig {
            prefix: "v".to_string(),
            version: VersionSection {
                major: 0,
                minor: 1,
                patch: 0,
                candidate: 0,
            },
            candidate: CandidateSection {
                promotion: "minor".to_string(),
                delimiter: "-rc".to_string(),
            },
            development: DevelopmentSection {
                promotion: "git_sha".to_string(),
                delimiter: "+".to_string(),
            },
        };
        
        Version {
            prefix: config.prefix.clone(),
            major: config.version.major,
            minor: config.version.minor,
            patch: config.version.patch,
            candidate: config.version.candidate,
            path: path.to_path_buf(),
            config,
        }
    }

    fn from_file(path: &Path) -> Result<Self, BumpError> {
        let content = fs::read_to_string(path).map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                BumpError::IoError(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{}", path.display()),
                ))
            } else {
                BumpError::IoError(err)
            }
        })?;

        let config: BumpConfig = toml::from_str(&content)?;

        Ok(Version {
            prefix: config.prefix.clone(),
            major: config.version.major,
            minor: config.version.minor,
            patch: config.version.patch,
            candidate: config.version.candidate,
            path: path.to_path_buf(),
            config,
        })
    }

    fn to_file(&self) -> Result<(), BumpError> {
        // Update the config with current version values
        let mut updated_config = self.config.clone();
        updated_config.prefix = self.prefix.clone();
        updated_config.version.major = self.major;
        updated_config.version.minor = self.minor;
        updated_config.version.patch = self.patch;
        updated_config.version.candidate = self.candidate;

        let toml_content = toml::to_string_pretty(&updated_config)
            .map_err(|e| BumpError::ParseError(format!("Failed to serialize TOML: {}", e)))?;
        
        // Add header comment to the TOML
        let content = format!(
            r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

{}
"#,
            toml_content
        );

        match fs::write(self.path.as_path(), content) {
            Ok(_) => Ok(()),
            Err(err) => Err(BumpError::IoError(err)),
        }
    }

    fn to_string(&self, bump_type: &BumpType) -> String {
        match bump_type {
            BumpType::Prefix(_) | BumpType::Point(_) | BumpType::Release => {
                format!(
                    "{}{}.{}.{}",
                    self.prefix, self.major, self.minor, self.patch
                )
            }
            BumpType::Candidate => format!(
                "{}{}.{}.{}{}{}",
                self.prefix, self.major, self.minor, self.patch, 
                self.config.candidate.delimiter, self.candidate
            ),
            // Useful for cmake and other tools
            BumpType::Base => format!("{}.{}.{}", self.major, self.minor, self.patch),
        }
    }

    fn from_string(version_str: &str, path: &Path) -> Result<Self, BumpError> {
        let re =
            Regex::new(r"^(?P<prefix>[a-zA-Z]*)(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?:-rc(?P<candidate>\d+))?")
                .unwrap();
        let caps = re
            .captures(version_str)
            .ok_or_else(|| BumpError::ParseError("invalid version format".to_string()))?;

        let prefix = caps
            .name("prefix")
            .map_or("v".to_string(), |m| m.as_str().to_string());
        let major = caps["major"]
            .parse()
            .map_err(|_| BumpError::ParseError("invalid MAJOR value".to_string()))?;
        let minor = caps["minor"]
            .parse()
            .map_err(|_| BumpError::ParseError("invalid MINOR value".to_string()))?;
        let patch = caps["patch"]
            .parse()
            .map_err(|_| BumpError::ParseError("invalid PATCH value".to_string()))?;
        let candidate = caps.name("candidate").map_or(Ok(0), |m| {
            m.as_str()
                .parse()
                .map_err(|_| BumpError::ParseError("invalid CANDIDATE value".to_string()))
        })?;

        // Create default config (in reality this should probably read from a config file)
        let config = BumpConfig {
            prefix: prefix.clone(),
            version: VersionSection {
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
        };

        Ok(Version {
            prefix,
            major,
            minor,
            patch,
            candidate,
            path: path.to_path_buf(),
            config,
        })
    }

    fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
        match bump_type {
            BumpType::Prefix(prefix) => {
                self.prefix = prefix.clone();
            }
            BumpType::Point(PointType::Major) => {
                self.major += 1;
                self.minor = 0;
                self.patch = 0;
                self.candidate = 0;
            }
            BumpType::Point(PointType::Minor) => {
                self.minor += 1;
                self.patch = 0;
                self.candidate = 0;
            }
            BumpType::Point(PointType::Patch) => {
                self.patch += 1;
                self.candidate = 0;
            }
            BumpType::Candidate => {
                if self.candidate > 0 {
                    self.candidate += 1;
                } else {
                    // Use promotion strategy from config
                    match self.config.candidate.promotion.as_str() {
                        "major" => {
                            self.major += 1;
                            self.minor = 0;
                            self.patch = 0;
                        }
                        "minor" => {
                            self.minor += 1;
                            self.patch = 0;
                        }
                        "patch" => {
                            self.patch += 1;
                        }
                        _ => {
                            // Default to minor if unrecognized strategy
                            self.minor += 1;
                            self.patch = 0;
                        }
                    }
                    self.candidate = 1; // start candidate at 1
                }
            }
            BumpType::Release => {
                // Release does not increment, just drops candidate and tags commit
                if self.candidate == 0 {
                    return Err(BumpError::LogicError(
                        "Cannot release without a candidate".to_string(),
                    ));
                }
                self.candidate = 0;
            }
            BumpType::Base => { /* won't happen */ }
        }
        Ok(())
    }
}

fn resolve_path(input_path: &str) -> PathBuf {
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

fn ensure_directory_exists(path: &Path) -> Result<(), BumpError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(BumpError::IoError)?;
        }
    }
    Ok(())
}

fn prompt_for_version(path: &Path) -> Result<Version, BumpError> {
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
                let config = BumpConfig {
                    prefix: "v".to_string(),
                    version: VersionSection {
                        major: parts[0],
                        minor: parts[1],
                        patch: parts[2],
                        candidate: 0,
                    },
                    candidate: CandidateSection {
                        promotion: "minor".to_string(),
                        delimiter: "-rc".to_string(),
                    },
                    development: DevelopmentSection {
                        promotion: "git_sha".to_string(),
                        delimiter: "+".to_string(),
                    },
                };
                
                Ok(Version {
                    prefix: "v".to_string(),
                    major: parts[0],
                    minor: parts[1],
                    patch: parts[2],
                    candidate: 0,
                    path: path.to_path_buf(),
                    config,
                })
            }
            _ => Err(BumpError::ParseError("invalid version format".to_string())),
        }
    }
}

fn get_version(matches: &ArgMatches) -> Result<Version, BumpError> {
    let version_file_path = matches
        .get_one::<String>("bumpfile")
        .expect("PATH not provided");
    let version_path = resolve_path(version_file_path);
    Version::from_file(&version_path)
}

fn get_bump_type(matches: &ArgMatches) -> Result<BumpType, BumpError> {
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

fn initialize(bumpfile: &str, prefix: &str) -> Result<(), BumpError> {
    let filepath = resolve_path(bumpfile);
    ensure_directory_exists(&filepath)?;

    // prompt for tag or manual
    let mut use_git_tag = String::new();
    println!("Use git tag for versioning? (y/n): ");
    io::stdin()
        .read_line(&mut use_git_tag)
        .map_err(BumpError::IoError)?;
    let use_git_tag = use_git_tag.trim().to_lowercase();

    if use_git_tag == "y" {
        match get_git_tag() {
            Ok(git_tag) => {
                println!("Found git tag: {git_tag}");
                let mut git_version = Version::from_string(&git_tag, &filepath)?;
                    git_version.prefix = prefix.to_string(); // Override prefix from CLI
                    git_version.to_file()?;
            }
            Err(err) => {
                return Err(err);
            }
        }
    } else {
        let mut version = prompt_for_version(&filepath)?;
        version.prefix = prefix.to_string(); // Override prefix from CLI
        version.to_file()?;
    }

    println!("Initialized new version file at '{}'", filepath.display());
    Ok(())
}

fn print(version: &Version, base: bool) {
    let bump_type = if base {
        BumpType::Base
    } else if version.candidate > 0 {
        BumpType::Candidate
    } else {
        // bump_type doesn't matter here
        BumpType::Point(PointType::Patch)
    };
    print!("{}", version.to_string(&bump_type));
}

fn apply(matches: &ArgMatches) -> Result<(), BumpError> {
    let mut version = get_version(matches)?;
    let bump_type = get_bump_type(matches)?;
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

fn is_git_repository() -> bool {
    ProcessCommand::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_git_tag() -> Result<String, BumpError> {
    let output = ProcessCommand::new("git")
        .args(["describe", "--exact-match", "--tags", "HEAD"])
        .output()
        .map_err(|e| {
            BumpError::Git(format!(
                "failed to run 'git describe --exact-match --tags HEAD': {e}"
            ))
        })?;

    if !output.status.success() {
        return Err(BumpError::Git("Current commit is not tagged".to_string()));
    }

    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(tag)
}

fn get_git_commit_sha() -> Result<String, BumpError> {
    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map_err(|e| BumpError::Git(format!("failed to run 'git rev-parse --short HEAD': {e}")))?;

    if !output.status.success() {
        return Err(BumpError::Git(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(sha)
}

fn get_git_branch() -> Result<String, BumpError> {
    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| BumpError::Git(format!("failed to run 'git rev-parse --abbrev-ref HEAD': {e}")))?;

    if !output.status.success() {
        return Err(BumpError::Git(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

fn get_development_suffix(version: &Version) -> Result<String, BumpError> {
    match version.config.development.promotion.as_str() {
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

fn generate(matches: &ArgMatches, lang: &Language) -> Result<(), BumpError> {
    if !is_git_repository() {
        return Err(BumpError::LogicError("Not in a git repository".to_string()));
    }

    let bumpfile = matches.get_one::<String>("bumpfile").unwrap();
    let version = Version::from_file(&resolve_path(bumpfile))?;
    let output_files: Vec<&String> = matches.get_many::<String>("output").unwrap().collect();

    let tagged = get_git_tag().is_ok();

    let version_string = match (tagged, version.candidate) {
        (true, 0) => format!("{}.{}.{}", version.major, version.minor, version.patch),
        (true, _) => format!(
            "{}.{}.{}{}{}",
            version.major, version.minor, version.patch,
            version.config.candidate.delimiter, version.candidate
        ),
        (false, 0) => format!(
            "{}.{}.{}{}{}",
            version.major,
            version.minor,
            version.patch,
            version.config.development.delimiter,
            get_development_suffix(&version)?
        ),
        (false, _) => format!(
            "{}.{}.{}{}{}{}{}",
            version.major,
            version.minor,
            version.patch,
            version.config.candidate.delimiter,
            version.candidate,
            version.config.development.delimiter,
            get_development_suffix(&version)?
        ),
    };

    for output_file in output_files {
        let output_path = Path::new(output_file);

        // Create directory if it doesn't exist (mkdir -p behavior)
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(BumpError::IoError)?;
        }
        lang::output_file(lang, &version, &version_string, output_path)?;
    }

    Ok(())
}

fn create_git_tag(version: &Version, message: Option<&str>) -> Result<(), BumpError> {
    if !is_git_repository() {
        return Err(BumpError::LogicError("Not in a git repository".to_string()));
    }

    // Create the conventional tag name based on version
    let tag_name = if version.candidate > 0 {
        format!(
            "{}{}.{}.{}{}{}",
            version.prefix, version.major, version.minor, version.patch,
            version.config.candidate.delimiter, version.candidate
        )
    } else {
        format!(
            "{}{}.{}.{}",
            version.prefix, version.major, version.minor, version.patch
        )
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
        let default_message = if version.candidate > 0 {
            format!(
                "chore(release): bump version to {}{}.{}.{}{}{}",
                version.prefix, version.major, version.minor, version.patch,
                version.config.candidate.delimiter, version.candidate
            )
        } else {
            format!(
                "chore(release): bump version to {}{}.{}.{}",
                version.prefix, version.major, version.minor, version.patch
            )
        };
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

fn tag_version(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = matches.get_one::<String>("bumpfile").unwrap();
    let version = Version::from_file(&resolve_path(bumpfile))?;
    let message = matches.get_one::<String>("message");

    create_git_tag(&version, message.map(|s| s.as_str()))
}

fn egress(result: Result<(), BumpError>) -> ExitCode {
    if let Err(err) = result {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let matches = Command::new("bump")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Semantic Version bumping with sane defaults")
        .subcommand(
            Command::new("init")
                .about("Initialize a new version file with default values")
                .arg(
                    Arg::new("bumpfile")
                        .value_name("bumpfile")
                        .value_parser(clap::value_parser!(String))
                        .default_value("bump.toml")
                        .help("Path to the bumpfile to initialize")
                )
                .arg(
                    Arg::new("prefix")
                        .long("prefix")
                        .value_name("PREFIX")
                        .value_parser(clap::value_parser!(String))
                        .default_value("v")
                        .help("Prefix for version tags (e.g., 'v', 'release-', or empty string)")
                )
        )
        .subcommand(
            Command::new("gen")
                .about("Generate header files using git tag detection")
                .arg(
                    Arg::new("bumpfile")
                        .value_name("bumpfile")
                        .value_parser(clap::value_parser!(String))
                        .required(true)
                        .help("Path to the bumpfile to read version from")
                )
                .arg(
                    Arg::new("lang")
                        .short('l')
                        .long("lang")
                        .value_name("LANG")
                        .value_parser(clap::builder::PossibleValuesParser::new(["c", "java", "csharp", "go"]))
                        .num_args(1)
                        .required(true)
                        .help("Programming language for output files")
                )
                .arg(
                    Arg::new("output")
                        .value_name("output")
                        .value_parser(clap::value_parser!(String))
                        .action(clap::ArgAction::Append)
                        .required(true)
                        .help("Output files for header generation (multiple files can be generated from a single bumpfile)")
                )
        )
        .subcommand(
            Command::new("tag")
                .about("Create a conventional git tag based on the current bumpfile version")
                .arg(
                    Arg::new("bumpfile")
                        .value_name("bumpfile")
                        .value_parser(clap::value_parser!(String))
                        .default_value("bump.toml")
                        .help("Path to the bumpfile to read version from")
                )
                .arg(
                    Arg::new("message")
                        .short('m')
                        .long("message")
                        .value_name("MESSAGE")
                        .value_parser(clap::value_parser!(String))
                        .help("Custom tag message (defaults to conventional commit format)")
                )
        )
        .arg(
            Arg::new("bumpfile")
                .value_name("PATH")
                .value_parser(clap::value_parser!(String))
                .default_value("bump.toml")
                .help("Path to the version file"),
        )
        .arg(
            Arg::new("print")
                .short('p')
                .long("print")
                .action(clap::ArgAction::SetTrue)
                .group("print-group")
                .help("Print version from PATH, without a newline. Useful in CI/CD applications"),
        )
        .arg(
            Arg::new("print-base")
                .short('b')
                .long("print-base")
                .action(clap::ArgAction::SetTrue)
                .group("print-group")
                .help("Print base version (no candidate suffix) from PATH, without a newline. Useful for CMake"),
        )
        .arg(
            Arg::new("prefix")
                .long("prefix")
                .value_name("PREFIX")
                .value_parser(clap::value_parser!(String))
                .num_args(1)
                .group("meta")
                .help("Prefix for version tags (e.g., 'v', 'release-', or empty string)")
        )
        .arg(
            Arg::new("major")
                .long("major")
                .action(clap::ArgAction::SetTrue)
                .group("point-release")
                .conflicts_with_all(["meta", "candidate-release", "print-group"])
                .help("Bump the major version"),
        )
        .arg(
            Arg::new("minor")
                .long("minor")
                .action(clap::ArgAction::SetTrue)
                .group("point-release")
                .conflicts_with_all(["meta", "candidate-release", "print-group"])
                .help("Bump the minor version"),
        )
        .arg(
            Arg::new("patch")
                .long("patch")
                .action(clap::ArgAction::SetTrue)
                .group("point-release")
                .conflicts_with_all(["meta", "candidate-release", "print-group"])
                .help("Bump the patch version"),
        )
        .arg(
            Arg::new("release")
                .long("release")
                .action(clap::ArgAction::SetTrue)
                .group("point-release")
                .conflicts_with_all(["meta", "candidate-release", "print-group"])
                .help("Drop candidacy and promote to release")
        )
        .arg(
            Arg::new("candidate")
                .long("candidate")
                .action(clap::ArgAction::SetTrue)
                .help("if in candidacy increments the candidate version, otherwise bump the minor version and set the rc to 1")
                .group("candidate-release")
                .conflicts_with_all(["point-release", "print-group"])
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            let bumpfile = sub_matches.get_one::<String>("bumpfile").unwrap();
            let prefix = sub_matches.get_one::<String>("prefix").unwrap();
            egress(initialize(bumpfile, prefix))
        }
        Some(("gen", sub_matches)) => {
            let lang_str = sub_matches
                .get_one::<String>("lang")
                .expect("LANG not provided");
            let lang = match Language::from_str(lang_str) {
                Some(l) => l,
                None => {
                    return egress(Err(BumpError::LogicError(format!("Invalid language specified: {lang_str}"))));
                }
            };
            egress(generate(sub_matches, &lang))
        }
        Some(("tag", sub_matches)) => {
            egress(tag_version(sub_matches))
        }
        _ => {
            if matches.contains_id("print-group") {
                let version = match get_version(&matches) {
                    Ok(v) => v,
                    Err(err) => {
                        return egress(Err(err));
                    }
                };
                print(&version, matches.get_flag("print-base"));
                ExitCode::SUCCESS
            } else if matches.contains_id("point-release")
                || matches.contains_id("candidate-release")
                || matches.get_one::<String>("prefix").is_some()
            {
                egress(apply(&matches))
            } else {
                return egress(Err(BumpError::LogicError("no action specified. Run with --help to see available options.".to_string())));
            }
        }
    }
}

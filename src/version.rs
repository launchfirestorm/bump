use crate::bump::{
    BumpError, BumpType, PointType,
    get_development_suffix,
    get_git_tag,
    is_git_repository,
    resolve_path,
    run_git,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSection {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub candidate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSection {
    pub promotion: String, // "minor", "major", "patch"
    pub delimiter: String, // "-rc"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentSection {
    pub promotion: String, // "git_sha", "branch", "full"
    pub delimiter: String, // "+"
}

// SemVer Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemVerConfig {
    pub prefix: String,
    pub timestamp: Option<String>,
    pub version: VersionSection,
    pub candidate: CandidateSection,
    pub development: DevelopmentSection,
}

// CalVer Structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalVerConflictSection {
    pub resolution: String, // "suffix" or "overwrite"
    pub suffix: u32,
    pub delimiter: String, // "-"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalVerConfig {
    pub prefix: String,
    pub format: String,
    pub conflict: CalVerConflictSection,
}

// Configuration enum - either SemVer or CalVer
#[derive(Debug, Clone)]
pub enum Config {
    SemVer(SemVerConfig),
    CalVer(CalVerConfig),
}

// Wrapper for TOML parsing - has [semver] section
#[derive(Debug, Deserialize)]
struct SemVerToml {
    semver: SemVerConfig,
}

// Wrapper for TOML parsing - has [calver] section
#[derive(Debug, Deserialize)]
struct CalVerToml {
    calver: CalVerConfig,
}

pub(crate) fn default_semver_config(
    prefix: String,
    major: u32,
    minor: u32,
    patch: u32,
    candidate: u32,
) -> Config {
    Config::SemVer(SemVerConfig {
        prefix,
        timestamp: None,
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
    })
}

pub(crate) fn default_calver_config(prefix: String) -> Config {
    Config::CalVer(CalVerConfig {
        prefix,
        format: "%Y.%m.%d".to_string(),
        conflict: CalVerConflictSection {
            resolution: "suffix".to_string(),
            suffix: 0,
            delimiter: "-".to_string(),
        },
    })
}

// Version data - either SemVer or CalVer
#[derive(Debug, Clone)]
pub enum VersionType {
    SemVer {
        major: u32,
        minor: u32,
        patch: u32,
        candidate: u32,
    },
    CalVer {
        suffix: u32,
    },
}

#[derive(Debug)]
pub struct Version {
    pub prefix: String,
    pub timestamp: Option<String>,
    pub version_type: VersionType,
    pub path: PathBuf,
    pub config: Config,
}

fn get_time(format: &Option<String>) -> Option<String> {
    let now = chrono::Utc::now();
    format.as_ref().map(|fmt| now.format(fmt).to_string())
}

impl Version {
    pub fn default(path: &Path) -> Self {
        let config = default_semver_config("v".to_string(), 0, 1, 0, 0);

        match &config {
            Config::SemVer(semver_config) => Version {
                prefix: semver_config.prefix.clone(),
                timestamp: None,
                version_type: VersionType::SemVer {
                    major: semver_config.version.major,
                    minor: semver_config.version.minor,
                    patch: semver_config.version.patch,
                    candidate: semver_config.version.candidate,
                },
                path: path.to_path_buf(),
                config,
            },
            Config::CalVer(_) => unreachable!("default should use SemVer"),
        }
    }

    pub fn from_argmatches(matches: &clap::ArgMatches) -> Result<Self, BumpError> {
        let bumpfile = matches
            .get_one::<String>("bumpfile")
            .expect("bumpfile not provided");
        let path = resolve_path(bumpfile);
        Version::from_file(&path)
    }

    pub fn from_file(path: &Path) -> Result<Self, BumpError> {
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

        // Parse as Value to check which sections exist
        let toml_value: toml::Value = toml::from_str(&content)?;
        
        let has_semver = toml_value.get("semver").is_some();
        let has_calver = toml_value.get("calver").is_some();

        // Enforce mutual exclusivity
        if has_semver && has_calver {
            return Err(BumpError::ParseError(
                "Cannot have both [semver] and [calver] sections. Please comment out one.".to_string()
            ));
        }

        if !has_semver && !has_calver {
            return Err(BumpError::ParseError(
                "Must have either [semver] or [calver] section defined.".to_string()
            ));
        }

        if has_semver {
            // Parse SemVer config
            let semver_toml: SemVerToml = toml::from_str(&content)?;
            let semver_config = semver_toml.semver;

            // Validate development promotion strategy
            match semver_config.development.promotion.as_str() {
                "git_sha" | "branch" | "full" => (),
                _ => {
                    println!(
                        "invalid development promotion strategy: {}",
                        semver_config.development.promotion
                    );
                    println!("defaulting to git_sha");
                }
            }

            // Validate candidate promotion strategy
            match semver_config.candidate.promotion.as_str() {
                "minor" | "major" | "patch" => (),
                _ => {
                    println!(
                        "invalid candidate promotion strategy: {}",
                        semver_config.candidate.promotion
                    );
                    println!("defaulting to minor");
                }
            }

            Ok(Version {
                prefix: semver_config.prefix.clone(),
                timestamp: get_time(&semver_config.timestamp),
                version_type: VersionType::SemVer {
                    major: semver_config.version.major,
                    minor: semver_config.version.minor,
                    patch: semver_config.version.patch,
                    candidate: semver_config.version.candidate,
                },
                path: path.to_path_buf(),
                config: Config::SemVer(semver_config),
            })
        } else {
            // Parse CalVer config
            let calver_toml: CalVerToml = toml::from_str(&content)?;
            let calver_config = calver_toml.calver;

            // Validate conflict resolution strategy
            match calver_config.conflict.resolution.as_str() {
                "suffix" | "overwrite" => (),
                _ => {
                    println!(
                        "invalid conflict resolution strategy: {}",
                        calver_config.conflict.resolution
                    );
                    println!("defaulting to suffix");
                }
            }

            Ok(Version {
                prefix: calver_config.prefix.clone(),
                timestamp: None, // CalVer doesn't use separate timestamp
                version_type: VersionType::CalVer {
                    suffix: calver_config.conflict.suffix,
                },
                path: path.to_path_buf(),
                config: Config::CalVer(calver_config),
            })
        }
    }

    pub fn to_file(&self) -> Result<(), BumpError> {
        // Try to read existing file to preserve comments and formatting
        let original_content = fs::read_to_string(&self.path).unwrap_or_else(|_| {
            // If file doesn't exist, create default structure with header
            match &self.config {
                Config::SemVer(_) => r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

[semver]
prefix = "v"
timestamp = "%Y-%m-%d %H:%M:%S %Z"   # strftime syntax

# NOTE: This section is modified by the bump command
[semver.version]
major = 0
minor = 0
patch = 0
candidate = 0

[semver.candidate]
promotion = "minor"  # ["minor", "major", "patch"]
delimiter = "-rc"

# promotion strategies:
#  - git_sha ( 7 char sha1 of the current commit )
#  - branch ( append branch name )
#  - full ( <branch>_<sha1> )
[semver.development]
promotion = "git_sha"
delimiter = "+"
"#
                    .to_string(),
                Config::CalVer(_) => r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

[calver]
prefix = ""
format = "%Y.%m.%d"

[calver.conflict]
resolution = "suffix"  # overwrite | suffix
suffix = 0
delimiter = "-"
"#
                    .to_string(),
            }
        });

        // Parse the TOML document while preserving formatting
        let mut doc = original_content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {}", e)))?;

        match (&self.version_type, &self.config) {
            (VersionType::SemVer { major, minor, patch, candidate }, Config::SemVer(semver_config)) => {
                // Update SemVer values while preserving structure and comments
                doc["semver"]["prefix"] = value(&self.prefix);
                doc["semver"]["version"]["major"] = value(*major as i64);
                doc["semver"]["version"]["minor"] = value(*minor as i64);
                doc["semver"]["version"]["patch"] = value(*patch as i64);
                doc["semver"]["version"]["candidate"] = value(*candidate as i64);

                // Update candidate section if it exists
                if let Some(candidate_table) = doc.get_mut("semver")
                    .and_then(|s| s.get_mut("candidate"))
                    .and_then(|c| c.as_table_mut())
                {
                    candidate_table["promotion"] = value(&semver_config.candidate.promotion);
                    candidate_table["delimiter"] = value(&semver_config.candidate.delimiter);
                }

                // Update development section if it exists
                if let Some(dev_table) = doc.get_mut("semver")
                    .and_then(|s| s.get_mut("development"))
                    .and_then(|d| d.as_table_mut())
                {
                    dev_table["promotion"] = value(&semver_config.development.promotion);
                    dev_table["delimiter"] = value(&semver_config.development.delimiter);
                }
            }
            (VersionType::CalVer { suffix }, Config::CalVer(calver_config)) => {
                // Update CalVer values while preserving structure and comments
                doc["calver"]["prefix"] = value(&self.prefix);
                doc["calver"]["format"] = value(&calver_config.format);
                doc["calver"]["conflict"]["suffix"] = value(*suffix as i64);

                // Update conflict section if it exists
                if let Some(conflict_table) = doc.get_mut("calver")
                    .and_then(|c| c.get_mut("conflict"))
                    .and_then(|c| c.as_table_mut())
                {
                    conflict_table["resolution"] = value(&calver_config.conflict.resolution);
                    conflict_table["delimiter"] = value(&calver_config.conflict.delimiter);
                }
            }
            _ => unreachable!("Version type and config mismatch"),
        }

        // Write the updated document back to file
        match fs::write(self.path.as_path(), doc.to_string()) {
            Ok(_) => Ok(()),
            Err(err) => Err(BumpError::IoError(err)),
        }
    }

    pub fn to_string(&self, bump_type: &BumpType) -> String {
        match &self.version_type {
            VersionType::SemVer { major, minor, patch, candidate } => {
                match &self.config {
                    Config::SemVer(semver_config) => {
                        let base = format!(
                            "{}{}.{}.{}",
                            self.prefix, major, minor, patch
                        );
                        let candidate_str = format!(
                            "{}{}.{}.{}{}{}",
                            self.prefix,
                            major,
                            minor,
                            patch,
                            semver_config.candidate.delimiter,
                            candidate
                        );
                        match bump_type {
                            BumpType::Prefix(_) | BumpType::Point(_) | BumpType::Release => base,
                            BumpType::Candidate => candidate_str,
                            // Useful for cmake and other tools
                            BumpType::Base => format!("{}.{}.{}", major, minor, patch),
                        }
                    }
                    _ => unreachable!("SemVer version type must have SemVer config"),
                }
            }
            VersionType::CalVer { suffix } => {
                match &self.config {
                    Config::CalVer(calver_config) => {
                        let now = chrono::Utc::now();
                        let date_str = now.format(&calver_config.format).to_string();
                        if *suffix > 0 {
                            format!("{}{}{}{}", self.prefix, date_str, calver_config.conflict.delimiter, suffix)
                        } else {
                            format!("{}{}", self.prefix, date_str)
                        }
                    }
                    _ => unreachable!("CalVer version type must have CalVer config"),
                }
            }
        }
    }

    pub fn fully_qualified_string(&self) -> Result<String, BumpError> {
        match &self.version_type {
            VersionType::SemVer { major, minor, patch, candidate } => {
                match &self.config {
                    Config::SemVer(semver_config) => {
                        if !is_git_repository() {
                            // Not in a git repository - return base version without development suffix
                            if *candidate > 0 {
                                return Ok(format!(
                                    "{}{}.{}.{}{}{}",
                                    self.prefix,
                                    major,
                                    minor,
                                    patch,
                                    semver_config.candidate.delimiter,
                                    candidate
                                ));
                            } else {
                                return Ok(format!(
                                    "{}{}.{}.{}",
                                    self.prefix, major, minor, patch
                                ));
                            }
                        }

                        let tagged = get_git_tag(false).is_ok();
                        let base = format!(
                            "{}{}.{}.{}",
                            self.prefix, major, minor, patch
                        );
                        let candidate_str = format!(
                            "{}{}.{}.{}{}{}",
                            self.prefix,
                            major,
                            minor,
                            patch,
                            semver_config.candidate.delimiter,
                            candidate
                        );

                        let version_string = match (tagged, *candidate) {
                            (true, 0) => base,
                            (true, _) => candidate_str,
                            (false, 0) => format!(
                                "{}{}{}",
                                base,
                                semver_config.development.delimiter,
                                get_development_suffix(&self)?
                            ),
                            (false, _) => format!(
                                "{}{}{}",
                                candidate_str,
                                semver_config.development.delimiter,
                                get_development_suffix(&self)?
                            ),
                        };

                        Ok(version_string)
                    }
                    _ => unreachable!("SemVer version type must have SemVer config"),
                }
            }
            VersionType::CalVer { suffix } => {
                match &self.config {
                    Config::CalVer(calver_config) => {
                        let now = chrono::Utc::now();
                        let date_str = now.format(&calver_config.format).to_string();
                        if *suffix > 0 {
                            Ok(format!("{}{}{}{}", self.prefix, date_str, calver_config.conflict.delimiter, suffix))
                        } else {
                            Ok(format!("{}{}", self.prefix, date_str))
                        }
                    }
                    _ => unreachable!("CalVer version type must have CalVer config"),
                }
            }
        }
    }

    pub fn from_string(version_str: &str, path: &Path) -> Result<Self, BumpError> {
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

        let config = default_semver_config(prefix.clone(), major, minor, patch, candidate);

        Ok(Version {
            prefix,
            timestamp: None,
            version_type: VersionType::SemVer {
                major,
                minor,
                patch,
                candidate,
            },
            path: path.to_path_buf(),
            config,
        })
    }

    pub fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
        match &mut self.version_type {
            VersionType::SemVer { major, minor, patch, candidate } => {
                match &self.config {
                    Config::SemVer(semver_config) => {
                        self.timestamp = get_time(&semver_config.timestamp);
                        
                        match bump_type {
                            BumpType::Prefix(prefix) => {
                                self.prefix = prefix.clone();
                            }
                            BumpType::Point(PointType::Major) => {
                                *major += 1;
                                *minor = 0;
                                *patch = 0;
                                *candidate = 0;
                            }
                            BumpType::Point(PointType::Minor) => {
                                *minor += 1;
                                *patch = 0;
                                *candidate = 0;
                            }
                            BumpType::Point(PointType::Patch) => {
                                *patch += 1;
                                *candidate = 0;
                            }
                            BumpType::Candidate => {
                                if *candidate > 0 {
                                    *candidate += 1;
                                } else {
                                    // Use promotion strategy from config
                                    match semver_config.candidate.promotion.as_str() {
                                        "major" => {
                                            *major += 1;
                                            *minor = 0;
                                            *patch = 0;
                                        }
                                        "minor" => {
                                            *minor += 1;
                                            *patch = 0;
                                        }
                                        "patch" => {
                                            *patch += 1;
                                        }
                                        _ => {
                                            // Default to minor if unrecognized strategy
                                            *minor += 1;
                                            *patch = 0;
                                        }
                                    }
                                    *candidate = 1; // start candidate at 1
                                }
                            }
                            BumpType::Release => {
                                // Release does not increment, just drops candidate and tags commit
                                if *candidate == 0 {
                                    return Err(BumpError::LogicError(
                                        "Cannot release without a candidate".to_string(),
                                    ));
                                }
                                *candidate = 0;
                            }
                            BumpType::Base => { /* won't happen */ }
                        }
                        Ok(())
                    }
                    _ => unreachable!("SemVer version type must have SemVer config"),
                }
            }
            VersionType::CalVer { suffix } => {
                match &self.config {
                    Config::CalVer(calver_config) => {
                        // For CalVer, we always regenerate from current date
                        // Check if this date version already exists in git tags
                        let now = chrono::Utc::now();
                        let date_str = now.format(&calver_config.format).to_string();
                        let base_version = format!("{}{}", self.prefix, date_str);
                        
                        // Check if we're in a git repository and if the version exists
                        if is_git_repository() {
                            // Try to get all tags and see if any match today's date
                            match run_git("git tag") {
                                Ok(tags_output) => {
                                    let tags: Vec<&str> = tags_output.lines().collect();
                                    let mut existing_suffix = 0;
                                    
                                    // Find the highest suffix for today's date
                                    for tag in tags {
                                        if tag == base_version {
                                            existing_suffix = 0;
                                        } else if let Some(suffix_str) = tag.strip_prefix(&format!("{}{}", base_version, calver_config.conflict.delimiter)) {
                                            if let Ok(s) = suffix_str.parse::<u32>() {
                                                if s > existing_suffix {
                                                    existing_suffix = s;
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Handle conflict resolution
                                    match calver_config.conflict.resolution.as_str() {
                                        "suffix" => {
                                            // Increment the suffix
                                            *suffix = existing_suffix + 1;
                                        }
                                        "overwrite" => {
                                            // Keep suffix at 0 (no suffix)
                                            *suffix = 0;
                                        }
                                        _ => {
                                            // Default to suffix
                                            *suffix = existing_suffix + 1;
                                        }
                                    }
                                }
                                Err(_) => {
                                    // If we can't get tags, just use suffix 0
                                    *suffix = 0;
                                }
                            }
                        } else {
                            // Not in git repository, no conflict possible
                            *suffix = 0;
                        }
                        
                        Ok(())
                    }
                    _ => unreachable!("CalVer version type must have CalVer config"),
                }
            }
        }
    }
}

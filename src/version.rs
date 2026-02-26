use crate::bump::{
    BumpError, BumpType, PointType,
    get_development_suffix,
    get_git_tag,
    is_git_repository,
    resolve_path,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemVerFormatSection {
    pub prefix: String,
    pub delimiter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>, // strftime format
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemVerVersionSection {
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
    pub format: SemVerFormatSection,
    pub version: SemVerVersionSection,
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
pub struct CalVerFormatSection {
    pub prefix: String,
    pub delimiter: String,
    pub year: String,          // e.g., "%Y"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<String>, // e.g., "%m"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<String>,   // e.g., "%d"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<bool>,   // include minor component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub micro: Option<bool>,   // include micro component
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalVerVersionSection {
    pub year: String,           // e.g., "2026"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<String>,  // e.g., "02"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<String>,    // e.g., "25"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<u32>,     // numeric if format.minor is true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub micro: Option<u32>,     // numeric if format.micro is true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalVerConfig {
    pub format: CalVerFormatSection,
    pub version: CalVerVersionSection,
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
        format: SemVerFormatSection {
            prefix,
            delimiter: ".".to_string(),
            timestamp: Some("%Y-%m-%d %H:%M:%S %Z".to_string()),
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

pub(crate) fn default_calver_config(prefix: String) -> Config {
    let now = chrono::Utc::now();
    Config::CalVer(CalVerConfig {
        format: CalVerFormatSection {
            prefix,
            delimiter: ".".to_string(),
            year: "%Y".to_string(),
            month: Some("%m".to_string()),
            day: Some("%d".to_string()),
            minor: Some(false),
            micro: Some(false),
        },
        version: CalVerVersionSection {
            year: now.format("%Y").to_string(),
            month: Some(now.format("%m").to_string()),
            day: Some(now.format("%d").to_string()),
            minor: None,
            micro: None,
        },
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
                prefix: semver_config.format.prefix.clone(),
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
                prefix: semver_config.format.prefix.clone(),
                timestamp: get_time(&semver_config.format.timestamp),
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
                prefix: calver_config.format.prefix.clone(),
                timestamp: None, // CalVer doesn't use separate timestamp
                version_type: VersionType::CalVer {
                    suffix: calver_config.conflict.suffix,
                },
                path: path.to_path_buf(),
                config: Config::CalVer(calver_config),
            })
        }
    }

    pub fn file_init(&self) -> Result<(), BumpError> {
        let contents = match &self.config {
                Config::SemVer(_) => r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

[semver.format]
prefix = "v"
delimiter = "."
timestamp = "%Y-%m-%d %H:%M:%S %Z"   # [optional] strftime syntax for build timestamp

# NOTE: This section is modified by the bump command
[semver.version]
major = 0
minor = 0
patch = 0
candidate = 0

# Candidate promotion strategies:  (when creating first candidate)
#  - "major" : increment major, zero minor and patch
#  - "minor" : increment minor, zero patch
#  - "patch" : increment patch
[semver.candidate]
promotion = "minor"
delimiter = "-rc"

# Development suffix strategies:
#  - "git_sha" : append 7 char sha1 of the current commit (default)
#  - "branch"  : append the current git branch name
#  - "full"    : append <branch>_<sha1>
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

# format will drive version section below
# - remove optional fields to change format
# - for minor|micro, setting to false is the same as removing
[calver.format]
prefix = ""
delimiter = "."
year = "%Y"        # strftime 4 digit year
month = "%m"       # [optional] strftime zero padded month
day = "%d"         # [optional] strftime zero padded day
minor = false      # [optional] minor version number
micro = false      # [optional] micro version number

# NOTE: This section is modified by the bump command
[calver.version]
year = "2025"
month = "04"
day = "28"


# Conflict resolution when date matches existing version:
#  - "suffix"    : append numeric suffix (e.g., 2024.02.25-1)
#  - "overwrite" : reuse the same version
# NOTE: suffix is modified by the bump command
[calver.conflict]
resolution = "suffix"
suffix = 0
delimiter = "-"
"#
                    .to_string(),

    };
        fs::write(&self.path, contents).map_err(BumpError::IoError)
    }

    pub fn to_file(&self) -> Result<(), BumpError> {
        // Try to read existing file to preserve comments and formatting
        if !self.path.exists() {
            return Err(BumpError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{}", self.path.display()),
            )));
        }
        let original_content = fs::read_to_string(&self.path).map_err(BumpError::IoError)?;
        // Parse the TOML document while preserving formatting
        let mut doc = original_content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {}", e)))?;

        match (&self.version_type, &self.config) {
            (VersionType::SemVer { major, minor, patch, candidate }, Config::SemVer(semver_config)) => {
                // Update SemVer format section
                doc["semver"]["format"]["prefix"] = value(&semver_config.format.prefix);
                doc["semver"]["format"]["delimiter"] = value(&semver_config.format.delimiter);
                if let Some(ref timestamp) = semver_config.format.timestamp {
                    doc["semver"]["format"]["timestamp"] = value(timestamp);
                }
                
                // Update SemVer version section
                doc["semver"]["version"]["major"] = value(*major as i64);
                doc["semver"]["version"]["minor"] = value(*minor as i64);
                doc["semver"]["version"]["patch"] = value(*patch as i64);
                doc["semver"]["version"]["candidate"] = value(*candidate as i64);

                // Update candidate section
                doc["semver"]["candidate"]["promotion"] = value(&semver_config.candidate.promotion);
                doc["semver"]["candidate"]["delimiter"] = value(&semver_config.candidate.delimiter);

                // Update development section
                doc["semver"]["development"]["promotion"] = value(&semver_config.development.promotion);
                doc["semver"]["development"]["delimiter"] = value(&semver_config.development.delimiter);
            }
            (VersionType::CalVer { suffix }, Config::CalVer(calver_config)) => {
                // Update CalVer format section
                doc["calver"]["format"]["prefix"] = value(&calver_config.format.prefix);
                doc["calver"]["format"]["delimiter"] = value(&calver_config.format.delimiter);
                doc["calver"]["format"]["year"] = value(&calver_config.format.year);
                
                if let Some(ref month) = calver_config.format.month {
                    doc["calver"]["format"]["month"] = value(month);
                }
                if let Some(ref day) = calver_config.format.day {
                    doc["calver"]["format"]["day"] = value(day);
                }
                if let Some(minor) = calver_config.format.minor {
                    doc["calver"]["format"]["minor"] = value(minor);
                }
                if let Some(micro) = calver_config.format.micro {
                    doc["calver"]["format"]["micro"] = value(micro);
                }
                
                // Update CalVer version section
                doc["calver"]["version"]["year"] = value(&calver_config.version.year);
                
                if let Some(ref month) = calver_config.version.month {
                    doc["calver"]["version"]["month"] = value(month);
                }
                if let Some(ref day) = calver_config.version.day {
                    doc["calver"]["version"]["day"] = value(day);
                }
                if let Some(minor) = calver_config.version.minor {
                    doc["calver"]["version"]["minor"] = value(minor as i64);
                }
                if let Some(micro) = calver_config.version.micro {
                    doc["calver"]["version"]["micro"] = value(micro as i64);
                }
                
                // Update CalVer conflict section
                doc["calver"]["conflict"]["suffix"] = value(*suffix as i64);
                doc["calver"]["conflict"]["resolution"] = value(&calver_config.conflict.resolution);
                doc["calver"]["conflict"]["delimiter"] = value(&calver_config.conflict.delimiter);
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
                            BumpType::Calendar => base, // Shouldn't happen but return base
                        }
                    }
                    _ => unreachable!("SemVer version type must have SemVer config"),
                }
            }
            VersionType::CalVer { suffix } => {
                match &self.config {
                    Config::CalVer(calver_config) => {
                        // Build version from stored components
                        let mut parts = vec![calver_config.version.year.clone()];
                        
                        if let Some(ref month) = calver_config.version.month {
                            parts.push(month.clone());
                        }
                        if let Some(ref day) = calver_config.version.day {
                            parts.push(day.clone());
                        }
                        if let Some(minor) = calver_config.version.minor {
                            parts.push(minor.to_string());
                        }
                        if let Some(micro) = calver_config.version.micro {
                            parts.push(micro.to_string());
                        }
                        
                        let version_str = parts.join(&calver_config.format.delimiter);
                        
                        if *suffix > 0 {
                            format!("{}{}{}{}", calver_config.format.prefix, version_str, calver_config.conflict.delimiter, suffix)
                        } else {
                            format!("{}{}", calver_config.format.prefix, version_str)
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
                                get_development_suffix(self)?
                            ),
                            (false, _) => format!(
                                "{}{}{}",
                                candidate_str,
                                semver_config.development.delimiter,
                                get_development_suffix(self)?
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
                        // Build version from stored components
                        let mut parts = vec![calver_config.version.year.clone()];
                        
                        if let Some(ref month) = calver_config.version.month {
                            parts.push(month.clone());
                        }
                        if let Some(ref day) = calver_config.version.day {
                            parts.push(day.clone());
                        }
                        if let Some(minor) = calver_config.version.minor {
                            parts.push(minor.to_string());
                        }
                        if let Some(micro) = calver_config.version.micro {
                            parts.push(micro.to_string());
                        }
                        
                        let version_str = parts.join(&calver_config.format.delimiter);
                        
                        if *suffix > 0 {
                            Ok(format!("{}{}{}{}", calver_config.format.prefix, version_str, calver_config.conflict.delimiter, suffix))
                        } else {
                            Ok(format!("{}{}", calver_config.format.prefix, version_str))
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
                        self.timestamp = get_time(&semver_config.format.timestamp);
                        
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
                            BumpType::Calendar => {
                                return Err(BumpError::LogicError(
                                    "SemVer does not support --calendar bump".to_string()
                                ));
                            }
                            BumpType::Base => { /* won't happen */ }
                        }
                        Ok(())
                    }
                    _ => unreachable!("SemVer version type must have SemVer config"),
                }
            }
            VersionType::CalVer { suffix } => {
                match &mut self.config {
                    Config::CalVer(calver_config) => {
                        match bump_type {
                            BumpType::Calendar => {
                                // Get current date and format components
                                let now = chrono::Utc::now();
                                let new_year = now.format(&calver_config.format.year).to_string();
                                let new_month = calver_config.format.month.as_ref()
                                    .map(|fmt| now.format(fmt).to_string());
                                let new_day = calver_config.format.day.as_ref()
                                    .map(|fmt| now.format(fmt).to_string());
                                
                                // Compare with stored version to check for conflict
                                let is_same_date = new_year == calver_config.version.year
                                    && new_month == calver_config.version.month
                                    && new_day == calver_config.version.day;
                                
                                if is_same_date {
                                    // Same date - handle conflict resolution
                                    match calver_config.conflict.resolution.as_str() {
                                        "suffix" => {
                                            // Increment the suffix
                                            *suffix += 1;
                                        }
                                        "overwrite" => {
                                            // Keep suffix at current value (usually 0)
                                            // Don't increment
                                        }
                                        _ => {
                                            // Default to suffix
                                            *suffix += 1;
                                        }
                                    }
                                } else {
                                    // Different date - reset suffix
                                    *suffix = 0;
                                    
                                    // Update version section with new date
                                    calver_config.version.year = new_year;
                                    calver_config.version.month = new_month;
                                    calver_config.version.day = new_day;
                                    
                                    // Handle minor/micro if enabled in format
                                    if let Some(true) = calver_config.format.minor {
                                        // Increment or initialize minor
                                        calver_config.version.minor = Some(
                                            calver_config.version.minor.map_or(0, |v| v + 1)
                                        );
                                    } else {
                                        calver_config.version.minor = None;
                                    }
                                    
                                    if let Some(true) = calver_config.format.micro {
                                        // Increment or initialize micro
                                        calver_config.version.micro = Some(
                                            calver_config.version.micro.map_or(0, |v| v + 1)
                                        );
                                    } else {
                                        calver_config.version.micro = None;
                                    }
                                }
                            }
                            _ => {
                                return Err(BumpError::LogicError(
                                    "CalVer only supports --calendar bump type".to_string()
                                ));
                            }
                        }
                        Ok(())
                    }
                    _ => unreachable!("CalVer version type must have CalVer config"),
                }
            }
        }
    }
}

use crate::bump::{
    BumpError, BumpType, PointType,
    get_development_suffix,
    get_git_tag,
    is_git_repository,
    resolve_path,
};
use serde::{Deserialize, Serialize};
use std::{
    fs, io, path::{Path, PathBuf}
};
use toml_edit::{DocumentMut, value};

//   _________            ____   ____            
//  /   _____/ ____   ____\   \ /   /___________ 
//  \_____  \_/ __ \ /     \   Y   // __ \_  __ \
//  /        \  ___/|  Y Y  \     /\  ___/|  | \/
// /_______  /\___  >__|_|  /\___/  \___  >__|   
//         \/     \/      \/            \/       
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemVerFormat {
    pub prefix: String,
    pub delimiter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>, // strftime format
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemVerVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub candidate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemverCandidate {
    pub promotion: String, // "minor", "major", "patch"
    pub delimiter: String, // "-rc"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemVerDevelopment {
    pub promotion: String, // "git_sha", "branch", "full"
    pub delimiter: String, // "+"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemVer {
    pub format: SemVerFormat,
    pub version: SemVerVersion,
    pub candidate: SemverCandidate,
    pub development: SemVerDevelopment,
}

// _________        .______   ____            
// \_   ___ \_____  |  \   \ /   /___________ 
// /    \  \/\__  \ |  |\   Y   // __ \_  __ \
// \     \____/ __ \|  |_\     /\  ___/|  | \/
//  \______  (____  /____/\___/  \___  >__|   
//         \/     \/                 \/       
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalVerFormat {
    pub prefix: String,
    pub delimiter: String,
    pub year: String,          // e.g., "%Y"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<String>, // e.g., "%m"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<String>,   // e.g., "%d"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalVerConflict {
    pub revision: u32,
    pub delimiter: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalVerVersion {
    pub year: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalVer {
    pub format: CalVerFormat,
    pub version: CalVerVersion,
    pub conflict: CalVerConflict,
}


// Wrapper for TOML parsing - has [semver] section
#[derive(Debug, Deserialize)]
struct SemVerToml {
    semver: SemVer,
}

// Wrapper for TOML parsing - has [calver] section
#[derive(Debug, Deserialize)]
struct CalVerToml {
    calver: CalVer,
}

pub fn default_semver(
    prefix: &str,
    major: u32,
    minor: u32,
    patch: u32,
    candidate: u32,
) -> SemVer {
    SemVer {
        format: SemVerFormat {
            prefix: prefix.to_string(),
            delimiter: ".".to_string(),
            timestamp: Some("%Y-%m-%d %H:%M:%S %Z".to_string()),
        },
        version: SemVerVersion {
            major,
            minor,
            patch,
            candidate,
        },
        candidate: SemverCandidate {
            promotion: "minor".to_string(),
            delimiter: "-rc".to_string(),
        },
        development: SemVerDevelopment {
            promotion: "git_sha".to_string(),
            delimiter: "+".to_string(),
        },
    }
}

pub fn default_calver(prefix: &str) -> CalVer {
    let now = chrono::Utc::now();
    CalVer {
        format: CalVerFormat {
            prefix: prefix.to_string(),
            delimiter: ".".to_string(),
            year: "%Y".to_string(),
            month: Some("%m".to_string()),
            day: Some("%d".to_string()),
        },
        version: CalVerVersion {
            year: now.format("%Y").to_string(),
            month: Some(now.format("%m").to_string()),
            day: Some(now.format("%d").to_string()),
        },
        conflict: CalVerConflict {
            revision: 0,
            delimiter: "-".to_string(),
        },
    }
}

// Version data - either SemVer or CalVer
#[derive(Debug, Clone)]
pub enum VersionType {
    SemVer(SemVer),
    CalVer(CalVer),
}

#[derive(Debug)]
pub struct Version {
    pub version_type: VersionType,
    pub path: PathBuf,
}

impl Version {
    pub fn from_argmatches(matches: &clap::ArgMatches) -> Result<Self, BumpError> {
        let bumpfile = matches
            .get_one::<String>("bumpfile")
            .expect("bumpfile not provided");
        let path = resolve_path(bumpfile);
        Version::from_file(&path)
    }

    pub fn get_timestamp(&self) -> Result<String, BumpError> {
        match &self.version_type {
            VersionType::SemVer ( semver ) => {
                let now = chrono::Utc::now();
                let timestamp = match semver.format.timestamp.as_ref() {
                    Some(fmt) => now.format(fmt).to_string(),
                    None => return Err(BumpError::LogicError("Timestamp format not specified".to_string())),
                };
                Ok(timestamp)
            },
            VersionType::CalVer (_) => {
                Err(BumpError::LogicError(
                    "You are using calendar versioning! --print-with-timestamp is redundant.".to_string()
                ))
            }
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, BumpError> {
        let content = fs::read_to_string(path).map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                BumpError::LogicError(format!(
                    "Configuration file not found at '{}'. Create one with 'bump init' or 'bump init --calver'",
                    path.display()
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
            let semver_toml: SemVerToml = toml::from_str(&content)?;
            let semver = semver_toml.semver;

            // Validate development promotion strategy
            match semver.development.promotion.as_str() {
                "git_sha" | "branch" | "full" => (),
                _ => {
                    println!(
                        "invalid development promotion strategy: {}",
                        semver.development.promotion
                    );
                    println!("defaulting to git_sha");
                }
            }

            // Validate candidate promotion strategy
            match semver.candidate.promotion.as_str() {
                "minor" | "major" | "patch" => (),
                _ => {
                    println!(
                        "invalid candidate promotion strategy: {}",
                        semver.candidate.promotion
                    );
                    println!("defaulting to minor");
                }
            }

            Ok(Version {
                version_type: VersionType::SemVer(semver),
                path: path.to_path_buf(),
            })
        } else {
            let calver_toml: CalVerToml = toml::from_str(&content)?;
            let calver = calver_toml.calver;

            Ok(Version {
                version_type: VersionType::CalVer(calver),
                path: path.to_path_buf(),
            })
        }
    }

    pub fn file_init(&self) -> Result<(), BumpError> {
        let contents = match &self.version_type {
            VersionType::SemVer(semver) => {
                format!(
                    r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

[semver.format]
prefix = "{}"
delimiter = "{}"
timestamp = "{}"   # [optional] strftime syntax for build timestamp

# NOTE: This section is modified by the bump command
[semver.version]
major = {}
minor = {}
patch = {}
candidate = {}

# Candidate promotion strategies:  (when creating first candidate)
#  - "major" : increment major, zero minor and patch
#  - "minor" : increment minor, zero patch
#  - "patch" : increment patch
[semver.candidate]
promotion = "{}"
delimiter = "{}"

# Development suffix strategies:
#  - "git_sha" : append 7 char sha1 of the current commit (default)
#  - "branch"  : append the current git branch name
#  - "full"    : append <branch>_<sha1>
[semver.development]
promotion = "{}"
delimiter = "{}"
"#,
                    semver.format.prefix,
                    semver.format.delimiter,
                    semver.format.timestamp.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S %Z"),
                    semver.version.major,
                    semver.version.minor,
                    semver.version.patch,
                    semver.version.candidate,
                    semver.candidate.promotion,
                    semver.candidate.delimiter,
                    semver.development.promotion,
                    semver.development.delimiter,
                )
            }
            VersionType::CalVer(calver) => {
                format!(
                    r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

# format will drive version section below
# - remove optional fields to change format
[calver.format]
prefix = ""
delimiter = "{}"
year = "{}"        # strftime 4 digit year
month = "{}"       # [optional] strftime zero padded month
day = "{}"         # [optional] strftime zero padded day

# NOTE: This section is modified by the bump command
[calver.version]
year = "{}"
month = "{}"
day = "{}"


# Same-date revision counter (only shown in version string if > 0)
# NOTE: revision is modified by the bump command
[calver.conflict]
revision = {}
delimiter = "{}"
"#,
                    calver.format.delimiter,
                    calver.format.year,
                    calver.format.month.as_deref().unwrap_or("%m"),
                    calver.format.day.as_deref().unwrap_or("%d"),
                    calver.version.year,
                    calver.version.month.as_deref().unwrap_or("01"),
                    calver.version.day.as_deref().unwrap_or("01"),
                    calver.conflict.revision,
                    calver.conflict.delimiter,
                )
            }
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

        match &self.version_type {
            VersionType::SemVer ( semver) => {
                // Update SemVer format section
                doc["semver"]["format"]["prefix"] = value(&semver.format.prefix);
                doc["semver"]["format"]["delimiter"] = value(&semver.format.delimiter);
                if let Some(ref timestamp) = semver.format.timestamp {
                    doc["semver"]["format"]["timestamp"] = value(timestamp);
                }
                
                doc["semver"]["version"]["major"] = value(semver.version.major as i64);
                doc["semver"]["version"]["minor"] = value(semver.version.minor as i64);
                doc["semver"]["version"]["patch"] = value(semver.version.patch as i64);
                doc["semver"]["version"]["candidate"] = value(semver.version.candidate as i64);
                doc["semver"]["candidate"]["promotion"] = value(&semver.candidate.promotion);
                doc["semver"]["candidate"]["delimiter"] = value(&semver.candidate.delimiter);
                doc["semver"]["development"]["promotion"] = value(&semver.development.promotion);
                doc["semver"]["development"]["delimiter"] = value(&semver.development.delimiter);
            }
            VersionType::CalVer ( calver) => {
                // NOTE: We don't touch the format section - it's static configuration
                // Only the version section and revision are modified during bumps
                doc["calver"]["version"]["year"] = value(&calver.version.year);
                
                if let Some(ref month) = calver.version.month {
                    doc["calver"]["version"]["month"] = value(month);
                } else {
                    // Remove field if not present in config
                    if let Some(table) = doc["calver"]["version"].as_table_mut() {
                        table.remove("month");
                    }
                }
                if let Some(ref day) = calver.version.day {
                    doc["calver"]["version"]["day"] = value(day);
                } else {
                    // Remove field if not present in config
                    if let Some(table) = doc["calver"]["version"].as_table_mut() {
                        table.remove("day");
                    }
                }
                
                doc["calver"]["conflict"]["revision"] = value(calver.conflict.revision as i64);
                doc["calver"]["conflict"]["delimiter"] = value(&calver.conflict.delimiter);
            }
        }

        // Write the updated document back to file
        match fs::write(self.path.as_path(), doc.to_string()) {
            Ok(_) => Ok(()),
            Err(err) => Err(BumpError::IoError(err)),
        }
    }

    fn get_semver_string(semver: &SemVer) -> Result<String, BumpError> {
        if !is_git_repository() {
            // Not in a git repository - return base version without development suffix
            if semver.version.candidate > 0 {
                return Ok(format!(
                    "{}{}.{}.{}{}{}",
                    semver.format.prefix,
                    semver.version.major,
                    semver.version.minor,
                    semver.version.patch,
                    semver.candidate.delimiter,
                    semver.version.candidate
                ));
            } else {
                return Ok(format!(
                    "{}{}.{}.{}",
                    semver.format.prefix, 
                    semver.version.major, 
                    semver.version.minor, 
                    semver.version.patch
                ));
            }
        }

        let tagged = get_git_tag(false).is_ok();
        let base = format!(
            "{}{}.{}.{}",
            semver.format.prefix, semver.version.major, semver.version.minor, semver.version.patch
        );
        let candidate_str = format!(
            "{}{}.{}.{}{}{}",
            semver.format.prefix,
            semver.version.major,
            semver.version.minor,
            semver.version.patch,
            semver.candidate.delimiter,
            semver.version.candidate
        );

        let version_string = match (tagged, semver.version.candidate) {
            (true, 0) => base,
            (true, _) => candidate_str,
            (false, 0) => format!(
                "{}{}{}",
                base,
                semver.development.delimiter,
                get_development_suffix(&semver.development.promotion)?
            ),
            (false, _) => format!(
                "{}{}{}",
                candidate_str,
                semver.development.delimiter,
                get_development_suffix(&semver.development.promotion)?
            ),
        };

        Ok(version_string)
    }

    fn get_calver_string(calver: &CalVer) -> Result<String, BumpError> {
        // Build version from stored components
        let mut parts = vec![calver.version.year.clone()];
        
        if let Some(ref month) = calver.version.month {
            parts.push(month.clone());
        }
        if let Some(ref day) = calver.version.day {
            parts.push(day.clone());
        }
        
        let version_str = parts.join(&calver.format.delimiter);
        
        let base_version = format!("{}{}", calver.format.prefix, version_str);
        
        // Only show revision if > 0
        if calver.conflict.revision > 0 {
            Ok(format!(
                "{}{}{}",
                base_version,
                calver.conflict.delimiter,
                calver.conflict.revision
            ))
        } else {
            Ok(base_version)
        }
    }

    pub fn to_string(&self) -> Result<String, BumpError> {
        match &self.version_type {
            VersionType::SemVer ( semver ) => {
                Version::get_semver_string(semver)
            }
            VersionType::CalVer ( calver ) => {
                Version::get_calver_string(calver)
            }
        }
    }

    pub fn to_base_string(&self) -> Result<String, BumpError> {
        match &self.version_type {
            VersionType::SemVer ( semver ) => {
                Ok(format!(
                    "{}{}.{}.{}",
                    semver.format.prefix,
                    semver.version.major,
                    semver.version.minor,
                    semver.version.patch
                ))
            }
            VersionType::CalVer ( _ ) => {
                Err(BumpError::LogicError(
                    "base version only applies to semantic versioning".to_string()
                )) 
            }
        }
    }

    pub fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
        match &mut self.version_type {
            VersionType::SemVer ( semver ) => {
                match bump_type {
                    BumpType::Prefix(prefix) => {
                        semver.format.prefix = prefix.clone();
                    }
                    BumpType::Point(PointType::Major) => {
                        semver.version.major += 1;
                        semver.version.minor = 0;
                        semver.version.patch = 0;
                        semver.version.candidate = 0;
                    }
                    BumpType::Point(PointType::Minor) => {
                        semver.version.minor += 1;
                        semver.version.patch = 0;
                        semver.version.candidate = 0;
                    }
                    BumpType::Point(PointType::Patch) => {
                        semver.version.patch += 1;
                        semver.version.candidate = 0;
                    }
                    BumpType::Candidate => {
                        if semver.version.candidate > 0 {
                            semver.version.candidate += 1;
                        } else {
                            match semver.candidate.promotion.as_str() {
                                "major" => {
                                    semver.version.major += 1;
                                    semver.version.minor = 0;
                                    semver.version.patch = 0;
                                }
                                "minor" => {
                                    semver.version.minor += 1;
                                    semver.version.patch = 0;
                                }
                                "patch" => {
                                    semver.version.patch += 1;
                                }
                                _ => {
                                    // Default to minor if unrecognized strategy
                                    semver.version.minor += 1;
                                    semver.version.patch = 0;
                                }
                            }
                            semver.version.candidate = 1; // start candidate at 1
                        }
                    }
                    BumpType::Release => {
                        // Release does not increment, just drops candidate and tags commit
                        if semver.version.candidate == 0 {
                            return Err(BumpError::LogicError(
                                "Cannot release without a candidate".to_string(),
                            ));
                        }
                        semver.version.candidate = 0;
                    }
                    BumpType::Calendar => {
                        return Err(BumpError::LogicError(
                            "SemVer does not support --calendar bump".to_string()
                        ));
                    }
                }
                Ok(())
            }
            VersionType::CalVer ( calver ) => {
                match bump_type {
                    BumpType::Calendar => {
                        // Get current date and format components
                        let now = chrono::Utc::now();
                        let new_year = now.format(&calver.format.year).to_string();
                        let new_month = calver.format.month.as_ref()
                            .map(|fmt| now.format(fmt).to_string());
                        let new_day = calver.format.day.as_ref()
                            .map(|fmt| now.format(fmt).to_string());
                        
                        // Compare with stored version to check for same date
                        let is_same_date = new_year == calver.version.year
                            && new_month == calver.version.month
                            && new_day == calver.version.day;
                        
                        if is_same_date {
                            // Same date - increment revision
                            calver.conflict.revision += 1;
                        } else {
                            // Different date - update date and reset revision
                            calver.conflict.revision = 0;
                            calver.version.year = new_year;
                            calver.version.month = new_month;
                            calver.version.day = new_day;
                        }
                        Ok(())
                    }
                    _ => {
                        return Err(BumpError::LogicError(
                            "CalVer only supports --calendar bump type".to_string()
                        ));
                    }
                }
            }
        }
    }
}

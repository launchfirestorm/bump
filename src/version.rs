use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, value};
use crate::bump::{BumpError, BumpType, PointType};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub prefix: String,
    pub version: VersionSection,
    pub candidate: CandidateSection,
    pub development: DevelopmentSection,
}


#[derive(Debug)]
pub struct Version {
    pub prefix: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub candidate: u32, // will be zero for point-release
    pub path: PathBuf,
    pub config: Config,
}

impl Version {
    pub fn default(path: &Path) -> Self {
        let config = Config {
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

        let config: Config = toml::from_str(&content)?;

        match config.development.promotion {
            ref str if str == "git_sha" || str == "branch" || str == "full" => (),
            _ => {
                println!("invalid development promotion strategy: {}", config.development.promotion);
                println!("defaulting to git_sha");
            }
        }

        match config.candidate.promotion {
            ref str if str == "minor" || str == "major" || str == "patch" => (),
            _ => {
                println!("invalid candidate promotion strategy: {}", config.candidate.promotion);
                println!("defaulting to minor");
            }
        }

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

    pub fn to_file(&self) -> Result<(), BumpError> {
        // Try to read existing file to preserve comments and formatting
        let original_content = fs::read_to_string(&self.path).unwrap_or_else(|_| {
            // If file doesn't exist, create default structure with header
            r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)  
#
# https://github.com/launchfirestorm/bump

prefix = "v"

# NOTE: This section is modified by the bump command
[version]
major = 0
minor = 0
patch = 0
candidate = 0

[candidate]
promotion = "minor"  # ["minor", "major", "patch"]
delimiter = "-rc"

# promotion strategies:
#  - git_sha ( 7 char sha1 of the current commit )
#  - branch ( append branch name )
#  - full ( <branch>_<sha1> )
[development]
promotion = "git_sha"
delimiter = "+"
"#
            .to_string()
        });

        // Parse the TOML document while preserving formatting
        let mut doc = original_content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {}", e)))?;

        // Update the values while preserving structure and comments
        doc["prefix"] = value(&self.prefix);
        doc["version"]["major"] = value(self.major as i64);
        doc["version"]["minor"] = value(self.minor as i64);
        doc["version"]["patch"] = value(self.patch as i64);
        doc["version"]["candidate"] = value(self.candidate as i64);

        // Update candidate section if it exists
        if let Some(candidate_table) = doc.get_mut("candidate")
            && let Some(table) = candidate_table.as_table_mut() {
                table["promotion"] = value(&self.config.candidate.promotion);
                table["delimiter"] = value(&self.config.candidate.delimiter);
            }

        // Update development section if it exists
        if let Some(dev_table) = doc.get_mut("development")
            && let Some(table) = dev_table.as_table_mut() {
                table["promotion"] = value(&self.config.development.promotion);
                table["delimiter"] = value(&self.config.development.delimiter);
            }

        // Write the updated document back to file
        match fs::write(self.path.as_path(), doc.to_string()) {
            Ok(_) => Ok(()),
            Err(err) => Err(BumpError::IoError(err)),
        }
    }

    pub fn to_string(&self, bump_type: &BumpType) -> String {
        match bump_type {
            BumpType::Prefix(_) | BumpType::Point(_) | BumpType::Release => {
                format!(
                    "{}{}.{}.{}",
                    self.prefix, self.major, self.minor, self.patch
                )
            }
            BumpType::Candidate => format!(
                "{}{}.{}.{}{}{}",
                self.prefix,
                self.major,
                self.minor,
                self.patch,
                self.config.candidate.delimiter,
                self.candidate
            ),
            // Useful for cmake and other tools
            BumpType::Base => format!("{}.{}.{}", self.major, self.minor, self.patch),
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

        let default_config = Config {
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
            config: default_config,
        })
    }

    pub fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
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
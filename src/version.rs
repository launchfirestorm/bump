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
    pub timestamp: Option<String>,
    pub version: VersionSection,
    pub candidate: CandidateSection,
    pub development: DevelopmentSection,
}

pub(crate) fn default_config(
    prefix: String,
    major: u32,
    minor: u32,
    patch: u32,
    candidate: u32,
) -> Config {
    Config {
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
    }
}

#[derive(Debug)]
pub struct Version {
    pub prefix: String,
    pub timestamp: Option<String>,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub candidate: u32, // will be zero for point-release
    pub path: PathBuf,
    pub config: Config,
}

fn get_time(format: &Option<String>) -> Option<String> {
    let now = chrono::Utc::now();
    format.as_ref().map(|fmt| now.format(fmt).to_string())
}

impl Version {
    pub fn default(path: &Path) -> Self {
        let config = default_config("v".to_string(), 0, 1, 0, 0);

        Version {
            prefix: config.prefix.clone(),
            timestamp: None,
            major: config.version.major,
            minor: config.version.minor,
            patch: config.version.patch,
            candidate: config.version.candidate,
            path: path.to_path_buf(),
            config,
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

        let config: Config = toml::from_str(&content)?;

        match config.development.promotion {
            ref str if str == "git_sha" || str == "branch" || str == "full" => (),
            _ => {
                println!(
                    "invalid development promotion strategy: {}",
                    config.development.promotion
                );
                println!("defaulting to git_sha");
            }
        }

        match config.candidate.promotion {
            ref str if str == "minor" || str == "major" || str == "patch" => (),
            _ => {
                println!(
                    "invalid candidate promotion strategy: {}",
                    config.candidate.promotion
                );
                println!("defaulting to minor");
            }
        }

        Ok(Version {
            prefix: config.prefix.clone(),
            timestamp: get_time(&config.timestamp),
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
timestamp = "%Y-%m-%d %H:%M:%S %Z"   # strftime syntax

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
            && let Some(table) = candidate_table.as_table_mut()
        {
            table["promotion"] = value(&self.config.candidate.promotion);
            table["delimiter"] = value(&self.config.candidate.delimiter);
        }

        // Update development section if it exists
        if let Some(dev_table) = doc.get_mut("development")
            && let Some(table) = dev_table.as_table_mut()
        {
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
        let base = format!(
            "{}{}.{}.{}",
            self.prefix, self.major, self.minor, self.patch
        );
        let candidate = format!(
            "{}{}.{}.{}{}{}",
            self.prefix,
            self.major,
            self.minor,
            self.patch,
            self.config.candidate.delimiter,
            self.candidate
        );
        match bump_type {
            BumpType::Prefix(_) | BumpType::Point(_) | BumpType::Release => base,
            BumpType::Candidate => candidate,
            // Useful for cmake and other tools
            BumpType::Base => format!("{}.{}.{}", self.major, self.minor, self.patch),
        }
    }

    pub fn fully_qualified_string(
        &self,
        repo_path: Option<&Path>,
    ) -> Result<String, BumpError> {
        if !is_git_repository(repo_path) {
            return Err(BumpError::LogicError("Not in a git repository".to_string()));
        }

        let tagged = get_git_tag(false, repo_path).is_ok();
        let base = format!(
            "{}{}.{}.{}",
            self.prefix, self.major, self.minor, self.patch
        );
        let candidate = format!(
            "{}{}.{}.{}{}{}",
            self.prefix,
            self.major,
            self.minor,
            self.patch,
            self.config.candidate.delimiter,
            self.candidate
        );

        let version_string = match (tagged, self.candidate) {
            (true, 0) => base,
            (true, _) => candidate,
            (false, 0) => format!(
                "{}{}{}",
                base,
                self.config.development.delimiter,
                get_development_suffix(&self, repo_path)?
            ),
            (false, _) => format!(
                "{}{}{}",
                candidate,
                self.config.development.delimiter,
                get_development_suffix(&self, repo_path)?
            ),
        };

        Ok(version_string)
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

        let default_config = default_config(prefix.clone(), major, minor, patch, candidate);

        Ok(Version {
            prefix,
            timestamp: None,
            major,
            minor,
            patch,
            candidate,
            path: path.to_path_buf(),
            config: default_config,
        })
    }

    pub fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
        self.timestamp = get_time(&self.config.timestamp);
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

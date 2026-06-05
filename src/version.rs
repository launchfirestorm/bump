use crate::bump::{
    BumpError, 
    BumpType, 
    PrintType, 
    get_git_branch, get_git_commit_sha, is_git_repository,
};
use chrono::{Datelike};
use serde::{Deserialize, Serialize};
use std::{
    fs, io, path::{Path, PathBuf}
};
use toml_edit::{DocumentMut, value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampTable {
    pub format: String,
    pub last: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionTable {
    #[serde(rename = "type")]
    pub _type: String,
    pub prefix: String,
    pub delimiter: String,
    pub major: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTable {
    pub name: String,
    pub delimiter: String,
    pub distance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuffixTable {
    #[serde(rename = "type")]
    pub _type: String,
    pub delimiter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    #[serde(skip)]
    pub path: PathBuf,
    pub timestamp: TimestampTable,
    // #[serde(flatten)]
    pub version: VersionTable,
    pub phase: PhaseTable,
    pub suffix: SuffixTable,
}

impl Version {
    pub fn default(path: &PathBuf) -> Self {
        Version {
            path: path.clone(),
            timestamp: TimestampTable {
                format: "%Y-%m-%d %H:%M:%S".to_string(),
                last: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            },
            version: VersionTable {
                _type: "semver".to_string(),
                prefix: "v".to_string(),
                delimiter: ".".to_string(),
                major: 0,
                minor: Some(1),
                patch: Some(0),
            },
            phase: PhaseTable {
                name: "".to_string(),
                delimiter: "-".to_string(),
                distance: 0,
            },
            suffix: SuffixTable {
                _type: "git_sha".to_string(),
                delimiter: "+".to_string(),
            },
        }
    }

    pub fn create_file(&self) -> Result<(), BumpError> {
        let strftime = "%Y-%m-%d %H:%M:%S %Z";
        let now = chrono::Utc::now();
        let current_timestamp = now.format(strftime).to_string();
        let content = format!(r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)
#
# https://github.com/launchfirestorm/bump

[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"   # strftime syntax, used in file generation
last = "{}"

[version]
type = "semver"  # or "calver"
prefix = "v"
delimiter = "."
major = 0  
minor = 1  # [optional] field can be removed if not needed
patch = 0  # [optional] can be removed if not needed

[phase]  
name = ""
delimiter = "."
distance = 0

# suffix type:
#  - "git_sha"  : append 7 char sha1 of the current commit (default)
#  - "branch"   : append the current git branch name
[suffix]
type = "git_sha"
delimiter = "+"
        "#, current_timestamp);
        Ok(fs::write(&self.path, content).map_err(BumpError::IoError)?)
    }

    pub fn from_file(path: &Path) -> Result<Self, BumpError> {
        let content = fs::read_to_string(path).map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                BumpError::LogicError(format!(
                    "Configuration file not found at '{}'. Create one with 'bump init'",
                    path.display()
                ))
            } else {
                BumpError::IoError(err)
            }
        })?;

        let version_parsed: Version = match toml::from_str(&content) {
            Ok(v) => {
                let mut version: Version = v;
                version.path = path.to_path_buf();
                version
            },
            Err(err) => {
                if ".version" == content.lines().find(|line| line.trim() == ".version").unwrap_or("") {
                    return Err(BumpError::ParseError(format!(
                        "Failed to parse file '{}': {}. Detected old format with [*.version] table. Please update to the new format with [version.type] field. See https://github.com/launchfirestorm/bump",
                        path.display(),
                        err
                    )));
                }
                return Err(BumpError::ParseError(format!(
                    "Failed to parse version file '{}': {}",
                    path.display(),
                    err
                )));
            }
        };

        match version_parsed.version._type.as_str() {
            "semver"|"calver" => (),
            _ => {
                return Err(BumpError::ParseError(format!(
                    "Invalid version type '{}' in '{}'. Expected 'semver' or 'calver'.",
                    version_parsed.version._type,
                    path.display()
                )));
            }
        }

        match version_parsed.suffix._type.as_str() {
            "git_sha"|"branch" => (),
            _ => {
                return Err(BumpError::ParseError(format!(
                    "Invalid suffix type '{}' in '{}'. Expected 'git_sha' or 'branch'.",
                    version_parsed.suffix._type,
                    path.display()
                )));
            }
        }
        Ok(version_parsed)
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

        doc["timestamp"]["format"] = value(&self.timestamp.format);
        doc["timestamp"]["last"] = value(&self.timestamp.last);

        doc["version"]["type"] = value(&self.version._type);
        doc["version"]["prefix"] = value(&self.version.prefix);
        doc["version"]["delimiter"] = value(&self.version.delimiter);
        doc["version"]["major"] = value(self.version.major as i64);
        if self.version.minor.is_some() {
            doc["version"]["minor"] = value(self.version.minor.unwrap() as i64);
        } else {
            doc["version"].as_table_mut().unwrap().remove("minor");
        }
        if self.version.patch.is_some() {
            doc["version"]["patch"] = value(self.version.patch.unwrap() as i64);
        } else {
            doc["version"].as_table_mut().unwrap().remove("patch");
        }

        doc["phase"]["name"] = value(&self.phase.name);
        doc["phase"]["delimiter"] = value(&self.phase.delimiter);
        doc["phase"]["distance"] = value(self.phase.distance as i64);

        doc["suffix"]["type"] = value(&self.suffix._type);
        doc["suffix"]["delimiter"] = value(&self.suffix.delimiter);

        match fs::write(self.path.as_path(), doc.to_string()) {
            Ok(_) => Ok(()),
            Err(err) => Err(BumpError::IoError(err)),
        }
    }

    pub fn to_string(&self, print_type: &PrintType) -> Result<String, BumpError> {
        match print_type {
            PrintType::OnlyPrefix => Ok(self.version.prefix.clone()),
            PrintType::OnlyPhase => Ok(self.phase.name.clone()),
            PrintType::OnlyBase => Ok(self.get_base()),
            PrintType::Regular => Ok(format!(
                "{}{}{}",
                self.version.prefix,
                self.get_base(),
                self.get_phase()
            )),
            PrintType::NoPrefix => Ok(format!(
                "{}{}",
                self.get_base(),
                self.get_phase()
            )),
            PrintType::NoPhase => Ok(format!(
                "{}{}",
                self.version.prefix,
                self.get_base()
            )),
            PrintType::WithSuffix => Ok(format!(
                "{}{}{}{}",
                self.version.prefix,
                self.get_base(),
                self.get_phase(),
                self.get_suffix()?
            )),
            PrintType::WithTimestamp => Ok(format!(
                "{}{}{}{}",
                self.version.prefix,
                self.get_base(),
                self.get_phase(),
                format!("  {}", self.timestamp.last)
            )),
            PrintType::Full => Ok(format!(
                "{}{}{}{}{}",
                self.version.prefix,
                self.get_base(),
                self.get_phase(),
                self.get_suffix()?,
                format!("  {}", self.timestamp.last)
            )),
        }
    }

    fn get_base(&self) -> String {
        if self.version.minor.is_some() && self.version.patch.is_some() {
            format!(
                "{}{}{}{}{}",
                &self.version.major,
                &self.version.delimiter,
                &self.version.minor.unwrap(),
                &self.version.delimiter,
                &self.version.patch.unwrap()
            )
        } else if self.version.minor.is_some() {
            format!(
                "{}{}{}", 
                &self.version.major, 
                &self.version.delimiter, 
                &self.version.minor.unwrap()
            )
        } else {
            format!("{}", &self.version.major)
        }
    }

    // empty phase name means no phase (formal release)
    fn get_phase(&self) -> String {
        match self.version._type.as_str() {
            "semver" => {
                if self.phase.name.is_empty() {
                    "".to_string()
                } else if self.phase.distance > 0 {
                    format!(
                        "{}{}{}",
                        self.phase.name,
                        self.phase.delimiter,
                        self.phase.distance,
                    )
                } else {
                    format!("{}", self.phase.name)
                }

            },
            "calver" => {
                if self.phase.name.is_empty() && self.phase.distance > 0 {
                    format!(
                        "{}{}",
                        &self.phase.delimiter,
                        &self.phase.distance,
                    )
                } else if self.phase.name.is_empty() && self.phase.distance == 0 {
                    "".to_string()
                } else {
                    format!("{}{}{}", self.phase.name, self.phase.delimiter, self.phase.distance)
                }
            },
            _ => "".to_string(), // should never happen due to validation in from_file
        }
    }

    fn get_suffix(&self) -> Result<String, BumpError> {
        if !is_git_repository() {
            return Err(BumpError::Git("Not a git repository".to_string()));
        }
        match self.suffix._type.as_str() {
            "git_sha" => {
                let sha = get_git_commit_sha()?;
                Ok(format!("{}{}", self.suffix.delimiter, sha))
            }
            "branch" => {
                let branch = get_git_branch()?;
                Ok(format!("{}{}", self.suffix.delimiter, branch))
            }
            _ => Ok("".to_string()), // should never happen due to validation in from_file
        }
    }

    // // internal call to give `apply()` the version string without dev suffixes
    // pub fn to_root_string(&self, prefix: bool) -> Result<String, BumpError> {
    //     match &self.version_type {
    //         VersionType::SemVer ( semver ) => {
    //             if semver.version.candidate > 0 {
    //                 Ok(format!(
    //                     "{}{}.{}.{}{}{}",
    //                     if prefix { &semver.format.prefix } else { "" },
    //                     semver.version.major,
    //                     semver.version.minor,
    //                     semver.version.patch,
    //                     semver.candidate.delimiter,
    //                     semver.version.candidate
    //                 ))
    //             } else {
    //                 Ok(format!(
    //                     "{}{}.{}.{}",
    //                     if prefix { &semver.format.prefix } else { "" },
    //                     semver.version.major,
    //                     semver.version.minor,
    //                     semver.version.patch
    //                 ))
    //             }
    //         }
    //         VersionType::CalVer ( calver ) => {
    //             Version::get_calver_string(calver)
    //         }
    //     }
    // }

    pub fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
        let now = chrono::Utc::now();
        match bump_type {
            BumpType::Major => {
                self.version.major += 1;
                self.version.minor = if self.version.minor.is_some() { Some(0) } else { None };
                self.version.patch = if self.version.patch.is_some() { Some(0) } else { None };
                self.phase.name = "".to_string();
                self.phase.distance = 0;
            }
            BumpType::Minor => {
                self.version.minor = Some(self.version.minor.unwrap_or(0) + 1);
                self.version.patch = if self.version.patch.is_some() { Some(0) } else { None };
                self.phase.name = "".to_string();
                self.phase.distance = 0;
            }
            BumpType::Patch => {
                self.version.patch = if self.version.patch.is_some() { Some(self.version.patch.unwrap_or(0) + 1) } else { None };
                self.phase.name = "".to_string();
                self.phase.distance = 0;
            }
            BumpType::Phase(cli_phase_name) => {
                let current_phase = &self.phase.name;
                if *cli_phase_name == "__increment__" {
                    self.phase.distance += 1;
                } else if current_phase == cli_phase_name {
                    self.phase.distance += 1;
                } else {
                    // different phase, switch to it and reset distance
                    self.phase.name = cli_phase_name.clone();
                    self.phase.distance = 0;
                }
            }
            BumpType::Calendar => {
                if self.version._type != "calver" {
                    return Err(BumpError::LogicError(
                        "Calendar bump is only applicable version.type = \"calver\"".to_string()
                    ));
                }
                if now.year() as u32 == self.version.major &&
                   now.month() as u32 == self.version.minor.unwrap_or(0) &&
                   now.day() as u32 == self.version.patch.unwrap_or(0) {
                    // If the date hasn't changed, just increment the phase distance (if any)
                    self.phase.distance += 1;
                }
                self.version.major = now.year() as u32;
                if self.version.minor.is_some() {
                    self.version.minor = Some(now.month() as u32);
                }
                if self.version.patch.is_some() {
                    self.version.patch = Some(now.day() as u32);
                }
            }
        }
        // always update timestamp on bump
        self.timestamp.last = now.format(&self.timestamp.format).to_string();
        Ok(())
    }
}

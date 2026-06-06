use crate::bump::{
    BumpError, BumpType, PrintType, get_git_branch, get_git_commit_sha, is_git_repository,
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampTable {
    pub format: String,
    pub last: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionTable {
    pub mode: String,
    pub prefix: String,
    pub delimiter: String,

    // semver and calver share the same fields but with different meanings
    #[serde(alias = "year")]
    pub major: u32,

    #[serde(alias = "month")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<u32>,

    #[serde(alias = "day")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTable {
    pub prefix: String,
    pub name: String,
    pub delimiter: String,
    pub distance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuffixTable {
    pub mode: String,
    pub delimiter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    #[serde(skip)]
    pub path: PathBuf,
    pub timestamp: TimestampTable,
    #[allow(clippy::struct_field_names)]
    pub version: VersionTable,
    pub phase: PhaseTable,
    pub suffix: SuffixTable,
}

impl Version {
    pub fn default(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            timestamp: TimestampTable {
                format: "%Y-%m-%d %H:%M:%S".to_string(),
                last: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            },
            version: VersionTable {
                mode: "semver".to_string(),
                prefix: "v".to_string(),
                delimiter: ".".to_string(),
                major: 0,
                minor: Some(1),
                patch: Some(0),
            },
            phase: PhaseTable {
                prefix: "-".to_string(),
                name: String::new(),
                delimiter: "-".to_string(),
                distance: 0,
            },
            suffix: SuffixTable {
                mode: "git_sha".to_string(),
                delimiter: "+".to_string(),
            },
        }
    }

    pub fn create_file(&self) -> Result<(), BumpError> {
        let strftime = "%Y-%m-%d %H:%M:%S %Z";
        let now = chrono::Utc::now();
        let current_timestamp = now.format(strftime).to_string();
        let content = format!(
            r#"#  ____  __  __  __  __  ____ 
# (  _ \(  )(  )(  \/  )(  _ \
#  ) _ < )(__)(  )    (  )___/
# (____/(______)(_/\/\_)(__)
#
# https://github.com/launchfirestorm/bump

[timestamp]
format = "%Y-%m-%d %H:%M:%S %Z"   # strftime syntax, used in file generation
last = "{current_timestamp}"

# NOTE: some fields are modified by bump
#   - mode: "semver" or "calver"
#   - minor|patch: optional, can be removed if not needed
[version]
mode = "semver"
prefix = "v"
delimiter = "."
major = 0  
minor = 1
patch = 0

[phase]  
prefix = "-"
name = ""
delimiter = "-"
distance = 0

# suffix type:
#  - "git_sha"  : append 7 char sha1 of the current commit (default)
#  - "branch"   : append the current git branch name
[suffix]
mode = "git_sha"
delimiter = "+"
        "#
        );
        fs::write(&self.path, content).map_err(BumpError::IoError)
    }

    fn warn_mode_key_mismatch(path: &Path, content: &str) -> Result<(), BumpError> {
        let doc = content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

        let version = doc["version"].as_table().ok_or_else(|| {
            BumpError::ParseError(format!(
                "'version' table not found in {}, bump v7 has changed the format. \
                \nDownload bump v7 from https://github.com/launchfirestorm/bump \
                \nand recreate your bumpfile with the proper format with 'bump init'.",
                path.display()
            ))
        })?;

        let mode = version
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("semver");
        let has_calver_keys = ["year", "month", "day"]
            .iter()
            .any(|key| version.contains_key(key));

        if mode == "semver" && has_calver_keys {
            println!(
                "bump warning: [version].mode is semver, but found calver keys (year/month/day) in {}. \
                \nThey will be treated as major/minor/patch and rewritten on save.",
                path.display()
            );
        }

        Ok(())
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

        Self::warn_mode_key_mismatch(path, &content)?;

        let version_parsed: Self = match toml::from_str(&content) {
            Ok(v) => {
                let mut version: Self = v;
                version.path = path.to_path_buf();
                version
            }
            Err(err) => {
                if ".version"
                    == content
                        .lines()
                        .find(|line| line.trim() == ".version")
                        .unwrap_or("")
                {
                    return Err(BumpError::ParseError(format!(
                        "Failed to parse file '{}': {}. \
                        \nDetected old format with [*.version] table. \
                        \nPlease update [version.mode] to 'semver' or 'calver'. \
                        \nSee https://github.com/launchfirestorm/bump",
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

        match version_parsed.version.mode.as_str() {
            "semver" | "calver" => (),
            _ => {
                return Err(BumpError::ParseError(format!(
                    "Invalid version type '{}' in '{}'. Expected 'semver' or 'calver'.",
                    version_parsed.version.mode,
                    path.display()
                )));
            }
        }

        match version_parsed.suffix.mode.as_str() {
            "git_sha" | "branch" => (),
            _ => {
                return Err(BumpError::ParseError(format!(
                    "Invalid suffix type '{}' in '{}'. Expected 'git_sha' or 'branch'.",
                    version_parsed.suffix.mode,
                    path.display()
                )));
            }
        }
        Ok(version_parsed)
    }

    fn version_remap(&self, doc: &mut DocumentMut) {
        let Some(version_table) = doc["version"].as_table_mut() else {
            return;
        };

        let (major_key, minor_key, patch_key, old_major, old_minor, old_patch) =
            if self.version.mode == "calver" {
                ("year", "month", "day", "major", "minor", "patch")
            } else {
                ("major", "minor", "patch", "year", "month", "day")
            };

        version_table[major_key] = value(i64::from(self.version.major));
        version_table.remove(old_major);

        if let Some(minor) = self.version.minor {
            version_table[minor_key] = value(i64::from(minor));
        } else {
            version_table.remove(minor_key);
        }
        version_table.remove(old_minor);

        if let Some(patch) = self.version.patch {
            version_table[patch_key] = value(i64::from(patch));
        } else {
            version_table.remove(patch_key);
        }
        version_table.remove(old_patch);
    }

    pub fn to_file(&self) -> Result<(), BumpError> {
        // Try to read existing file to preserve comments and formatting
        if !self.path.exists() {
            return Err(BumpError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                self.path.display().to_string(),
            )));
        }
        let original_content = fs::read_to_string(&self.path).map_err(BumpError::IoError)?;
        // Parse the TOML document while preserving formatting
        let mut doc = original_content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

        doc["timestamp"]["format"] = value(&self.timestamp.format);
        doc["timestamp"]["last"] = value(&self.timestamp.last);

        doc["version"]["mode"] = value(&self.version.mode);
        doc["version"]["prefix"] = value(&self.version.prefix);
        doc["version"]["delimiter"] = value(&self.version.delimiter);
        self.version_remap(&mut doc);

        doc["phase"]["prefix"] = value(&self.phase.prefix);
        doc["phase"]["name"] = value(&self.phase.name);
        doc["phase"]["delimiter"] = value(&self.phase.delimiter);
        doc["phase"]["distance"] = value(i64::from(self.phase.distance));

        doc["suffix"]["mode"] = value(&self.suffix.mode);
        doc["suffix"]["delimiter"] = value(&self.suffix.delimiter);

        fs::write(self.path.as_path(), doc.to_string()).map_err(BumpError::IoError)
    }

    #[rustfmt::skip]
    pub fn to_string(&self, print_type: &PrintType) -> Result<String, BumpError> {
        let prefix = &self.version.prefix;
        let base = self.get_base();
        let phase = self.get_phase();

        match print_type {
            PrintType::OnlyPrefix =>    Ok(prefix.clone()),
            PrintType::OnlyPhase =>     Ok(self.phase.name.clone()),
            PrintType::OnlyBase =>      Ok(base),
            PrintType::Regular =>       Ok(format!("{}{}{}", prefix, base, phase)),
            PrintType::NoPrefix =>      Ok(format!("{}{}", base, phase)),
            PrintType::NoPhase =>       Ok(format!("{}{}", prefix, base)),
            PrintType::WithSuffix =>    Ok(format!("{}{}{}{}", prefix, base, phase, self.get_suffix()?)),
            PrintType::WithTimestamp => Ok(format!("{}{}{}  {}", prefix, base, phase, self.timestamp.last)),
            PrintType::Full =>          Ok(format!("{}{}{}{}  {}", prefix, base, phase, self.get_suffix()?, self.timestamp.last)),
        }
    }

    fn format_component(&self, n: u32) -> String {
        if self.version.mode == "calver" {
            format!("{n:02}")
        } else {
            n.to_string()
        }
    }

    fn get_base(&self) -> String {
        match (self.version.minor, self.version.patch) {
            (Some(minor), Some(patch)) => format!(
                "{}{}{}{}{}",
                self.version.major,
                self.version.delimiter,
                self.format_component(minor),
                self.version.delimiter,
                self.format_component(patch),
            ),
            (Some(minor), None) => format!(
                "{}{}{}",
                self.version.major,
                self.version.delimiter,
                self.format_component(minor),
            ),
            (None, Some(patch)) => format!(
                "{}{}{}",
                self.version.major,
                self.version.delimiter,
                self.format_component(patch),
            ),
            _ => self.version.major.to_string(),
        }
    }

    // empty phase name means no phase (formal release)
    fn get_phase(&self) -> String {
        if self.phase.name.is_empty() && self.phase.distance == 0 {
            String::new()
        } else if self.phase.name.is_empty() && self.phase.distance > 0 {
            format!("{}{}", self.phase.prefix, self.phase.distance)
        } else if self.phase.distance == 0 {
            format!("{}{}", self.phase.prefix, self.phase.name,)
        } else {
            format!(
                "{}{}{}{}",
                self.phase.prefix, self.phase.name, self.phase.delimiter, self.phase.distance
            )
        }
    }

    fn get_suffix(&self) -> Result<String, BumpError> {
        if !is_git_repository() {
            return Err(BumpError::Git("Not a git repository".to_string()));
        }
        match self.suffix.mode.as_str() {
            "git_sha" => {
                let sha = get_git_commit_sha()?;
                Ok(format!("{}{}", self.suffix.delimiter, sha))
            }
            "branch" => {
                let branch = get_git_branch()?;
                Ok(format!("{}{}", self.suffix.delimiter, branch))
            }
            _ => Ok(String::new()), // should never happen due to validation in from_file
        }
    }

    fn right_mode(&self, expected_mode: &str) -> Result<(), BumpError> {
        if self.version.mode == expected_mode {
            Ok(())
        } else {
            Err(BumpError::LogicError(format!(
                "Operation only valid for version.type = '{expected_mode}'"
            )))
        }
    }

    pub fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
        let now = chrono::Utc::now();
        match bump_type {
            BumpType::Major => {
                self.right_mode("semver")?;
                self.version.major += 1;
                self.version.minor = self.version.minor.map(|_| 0);
                self.version.patch = self.version.patch.map(|_| 0);
                self.phase.name = String::new();
                self.phase.distance = 0;
            }
            BumpType::Minor => {
                self.right_mode("semver")?;
                self.version.minor = self.version.minor.map(|m| m + 1);
                self.version.patch = self.version.patch.map(|_| 0);
                self.phase.name = String::new();
                self.phase.distance = 0;
            }
            BumpType::Patch => {
                self.right_mode("semver")?;
                self.version.patch = self.version.patch.map(|p| p + 1);
                self.phase.name = String::new();
                self.phase.distance = 0;
            }
            BumpType::Phase(cli_phase_name) => {
                // BOTH modes have phases
                if cli_phase_name == &self.phase.name {
                    // same phase, just increment distance
                    self.phase.distance += 1;
                } else if *cli_phase_name != "__increment__" {
                    // different phase, switch to it and set distance to 1
                    self.phase.name.clone_from(cli_phase_name);
                    self.phase.distance = 1;
                } else {
                    // no arg just increment distance
                    self.phase.distance += 1;
                }
            }
            BumpType::Calendar => {
                self.right_mode("calver")?;
                if now.year().cast_unsigned() == self.version.major
                    && now.month() == self.version.minor.unwrap_or(0)
                    && now.day() == self.version.patch.unwrap_or(0)
                {
                    // If the date hasn't changed, just increment the phase distance (if any)
                    self.phase.distance += 1;
                } else {
                    self.version.major = now.year().cast_unsigned();
                    self.version.minor = self.version.minor.map(|_| now.month());
                    self.version.patch = self.version.patch.map(|_| now.day());
                }
            }
        }
        // always update timestamp on bump
        self.timestamp.last = now.format(&self.timestamp.format).to_string();
        Ok(())
    }
}

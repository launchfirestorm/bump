use crate::bump::{BumpError, BumpType};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionMode {
    Semver,
    Calver,
}

impl VersionMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Semver => "semver",
            Self::Calver => "calver",
        }
    }
}

impl fmt::Display for VersionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuffixMode {
    #[serde(rename = "git_sha")]
    GitSha,
    Branch,
}

impl SuffixMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GitSha => "git_sha",
            Self::Branch => "branch",
        }
    }

    pub fn parse(value: &str) -> Result<Self, BumpError> {
        match value {
            "git_sha" => Ok(Self::GitSha),
            "branch" => Ok(Self::Branch),
            _ => Err(BumpError::LogicError(format!(
                "Invalid suffix mode: '{value}'. Expected 'git_sha' or 'branch'."
            ))),
        }
    }
}

impl fmt::Display for SuffixMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LabelPosition {
    BeforePrefix,
    AfterPrefix,
    BeforeBase,
    AfterBase,
    BeforePhase,
    AfterPhase,
}

impl LabelPosition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeforePrefix => "before-prefix",
            Self::AfterPrefix => "after-prefix",
            Self::BeforeBase => "before-base",
            Self::AfterBase => "after-base",
            Self::BeforePhase => "before-phase",
            Self::AfterPhase => "after-phase",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp {
    pub format: String,
    pub last: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base {
    pub mode: VersionMode,
    pub delimiter: String,

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
pub struct Phase {
    pub separator: String,
    pub name: String,
    pub delimiter: String,
    pub distance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suffix {
    pub mode: SuffixMode,
    pub separator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub position: LabelPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    #[serde(skip)]
    pub path: PathBuf,
    pub prefix: String,
    pub timestamp: Timestamp,
    pub base: Base,
    pub phase: Phase,
    pub suffix: Suffix,
    pub label: Label,
}

const INIT_TEMPLATE_TIMESTAMP: &str = "1970-01-01 00:00:00 UTC";

impl Version {
    pub fn default(path: &Path) -> Self {
        let content = include_str!("templates/bump.toml")
            .replace("{timestamp}", INIT_TEMPLATE_TIMESTAMP);
        let mut version: Self = toml::from_str(&content).expect("init template must deserialize");
        version.path = path.to_path_buf();
        version.timestamp.last = chrono::Utc::now()
            .format(&version.timestamp.format)
            .to_string();
        version
    }

    pub fn create_file(&self) -> Result<(), BumpError> {
        let current_timestamp = chrono::Utc::now()
            .format(&self.timestamp.format)
            .to_string();
        let content =
            include_str!("templates/bump.toml").replace("{timestamp}", &current_timestamp);
        fs::write(&self.path, content).map_err(BumpError::IoError)
    }

    fn warn_mode_key_mismatch(path: &Path, content: &str) -> Result<(), BumpError> {
        let doc = content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

        let base = doc
            .get("base")
            .and_then(|item| item.as_table())
            .ok_or_else(|| {
                BumpError::ParseError(format!(
                    "'base' table not found in {}. \
                Recreate your bumpfile with 'bump init'.",
                    path.display()
                ))
            })?;

        let mode = base
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or(VersionMode::Semver.as_str());
        let has_calver_keys = ["year", "month", "day"]
            .iter()
            .any(|key| base.contains_key(key));

        if mode == VersionMode::Semver.as_str() && has_calver_keys {
            println!(
                "bump warning: [base].mode is semver, but found calver keys (year/month/day) in {}. \
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
                let mut version: Version = v;
                version.path = path.to_path_buf();
                version
            }
            Err(err) => {
                return Err(BumpError::ParseError(format!(
                    "Failed to parse version file '{}': {}. \
                    Recreate your bumpfile with 'bump init'.",
                    path.display(),
                    err
                )));
            }
        };

        Ok(version_parsed)
    }

    fn base_remap(&self, doc: &mut DocumentMut) {
        let Some(base_table) = doc["base"].as_table_mut() else {
            return;
        };

        let (major_key, minor_key, patch_key, old_major, old_minor, old_patch) =
            if self.base.mode == VersionMode::Calver {
                ("year", "month", "day", "major", "minor", "patch")
            } else {
                ("major", "minor", "patch", "year", "month", "day")
            };

        base_table[major_key] = value(i64::from(self.base.major));
        base_table.remove(old_major);

        if let Some(minor) = self.base.minor {
            base_table[minor_key] = value(i64::from(minor));
        } else {
            base_table.remove(minor_key);
        }
        base_table.remove(old_minor);

        if let Some(patch) = self.base.patch {
            base_table[patch_key] = value(i64::from(patch));
        } else {
            base_table.remove(patch_key);
        }
        base_table.remove(old_patch);
    }

    pub fn to_file(&self) -> Result<(), BumpError> {
        if !self.path.exists() {
            return Err(BumpError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                self.path.display().to_string(),
            )));
        }
        let original_content = fs::read_to_string(&self.path).map_err(BumpError::IoError)?;
        let mut doc = original_content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

        doc["prefix"] = value(&self.prefix);
        doc["timestamp"]["format"] = value(&self.timestamp.format);
        doc["timestamp"]["last"] = value(&self.timestamp.last);
        doc["base"]["mode"] = value(self.base.mode.as_str());
        doc["base"]["delimiter"] = value(&self.base.delimiter);
        self.base_remap(&mut doc);
        doc["phase"]["separator"] = value(&self.phase.separator);
        doc["phase"]["name"] = value(&self.phase.name);
        doc["phase"]["delimiter"] = value(&self.phase.delimiter);
        doc["phase"]["distance"] = value(i64::from(self.phase.distance));
        doc["suffix"]["mode"] = value(self.suffix.mode.as_str());
        doc["suffix"]["separator"] = value(&self.suffix.separator);
        doc["label"]["position"] = value(self.label.position.as_str());

        fs::write(self.path.as_path(), doc.to_string()).map_err(BumpError::IoError)
    }

    fn right_mode(&self, expected_mode: VersionMode) -> Result<(), BumpError> {
        if self.base.mode == expected_mode {
            Ok(())
        } else {
            Err(BumpError::LogicError(format!(
                "Operation only valid for version.type = '{}'",
                expected_mode.as_str()
            )))
        }
    }

    fn clear_phase(&mut self) {
        self.phase.name = String::new();
        self.phase.distance = 0;
    }

    pub fn bump(&mut self, bump_type: &BumpType) -> Result<(), BumpError> {
        let now = chrono::Utc::now();
        match bump_type {
            BumpType::Major => {
                self.right_mode(VersionMode::Semver)?;
                self.base.major += 1;
                self.base.minor = self.base.minor.map(|_| 0);
                self.base.patch = self.base.patch.map(|_| 0);
                self.clear_phase();
            }
            BumpType::Minor => {
                self.right_mode(VersionMode::Semver)?;
                self.base.minor = self.base.minor.map(|m| m + 1);
                self.base.patch = self.base.patch.map(|_| 0);
                self.clear_phase();
            }
            BumpType::Patch => {
                self.right_mode(VersionMode::Semver)?;
                self.base.patch = self.base.patch.map(|p| p + 1);
                self.clear_phase();
            }
            BumpType::Phase(cli_phase_name) => {
                if cli_phase_name == &self.phase.name {
                    self.phase.distance += 1;
                } else if *cli_phase_name != "__increment__" {
                    self.phase.name.clone_from(cli_phase_name);
                    self.phase.distance = 1;
                } else {
                    self.phase.distance += 1;
                }
            }
            BumpType::Calendar => {
                self.right_mode(VersionMode::Calver)?;
                if now.year().cast_unsigned() == self.base.major
                    && now.month() == self.base.minor.unwrap_or(0)
                    && now.day() == self.base.patch.unwrap_or(0)
                {
                    self.phase.distance += 1;
                } else {
                    self.base.major = now.year().cast_unsigned();
                    self.base.minor = self.base.minor.map(|_| now.month());
                    self.base.patch = self.base.patch.map(|_| now.day());
                }
            }
        }
        self.timestamp.last = now.format(&self.timestamp.format).to_string();
        Ok(())
    }
}

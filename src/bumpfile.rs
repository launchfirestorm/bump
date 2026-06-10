use crate::bump::{BumpError, ensure_directory_exists};
use crate::version::{Version, VersionMode};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Table, value};

const INIT_TEMPLATE_TIMESTAMP: &str = "1970-01-01 00:00:00 UTC";

pub struct BumpFile {
    path: PathBuf,
    doc: DocumentMut,
}

fn bumpfile_parse_error(path: &Path, message: impl fmt::Display) -> BumpError {
    BumpError::ParseError(format!(
        "{message} in {}. Recreate your bumpfile with 'bump init'.",
        path.display()
    ))
}

fn table<'a>(doc: &'a DocumentMut, section: &str, path: &Path) -> Result<&'a Table, BumpError> {
    doc.get(section)
        .and_then(|item| item.as_table())
        .ok_or_else(|| bumpfile_parse_error(path, format!("'{section}' table not found")))
}

fn table_mut<'a>(
    doc: &'a mut DocumentMut,
    section: &str,
    path: &Path,
) -> Result<&'a mut Table, BumpError> {
    doc.get_mut(section)
        .and_then(|item| item.as_table_mut())
        .ok_or_else(|| bumpfile_parse_error(path, format!("'{section}' table not found")))
}

fn require_key(table: &Table, key: &str, section: &str, path: &Path) -> Result<(), BumpError> {
    if table.contains_key(key) {
        Ok(())
    } else {
        Err(bumpfile_parse_error(
            path,
            format!("Expected key '{key}' not found in [{section}]"),
        ))
    }
}

fn set_top_str(doc: &mut DocumentMut, key: &str, val: &str, path: &Path) -> Result<(), BumpError> {
    require_key(doc, key, "(root)", path)?;
    doc[key] = value(val);
    Ok(())
}

fn set_str(
    table: &mut Table,
    key: &str,
    val: &str,
    section: &str,
    path: &Path,
) -> Result<(), BumpError> {
    require_key(table, key, section, path)?;
    table[key] = value(val);
    Ok(())
}

fn set_i64(
    table: &mut Table,
    key: &str,
    val: i64,
    section: &str,
    path: &Path,
) -> Result<(), BumpError> {
    require_key(table, key, section, path)?;
    table[key] = value(val);
    Ok(())
}

fn warn_mode_key_mismatch(path: &Path, content: &str) -> Result<(), BumpError> {
    let doc = content
        .parse::<DocumentMut>()
        .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

    let base = table(&doc, "base", path)?;

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

fn read_doc(path: &Path) -> Result<(PathBuf, DocumentMut), BumpError> {
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

    warn_mode_key_mismatch(path, &content)?;

    let doc = content
        .parse::<DocumentMut>()
        .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

    Ok((path.to_path_buf(), doc))
}

fn write_base(doc: &mut DocumentMut, version: &Version, path: &Path) -> Result<(), BumpError> {
    let base = table_mut(doc, "base", path)?;

    set_str(base, "mode", version.base.mode.as_str(), "base", path)?;
    set_str(base, "delimiter", &version.base.delimiter, "base", path)?;

    let (major_key, minor_key, patch_key, old_major, old_minor, old_patch) =
        if version.base.mode == VersionMode::Calver {
            ("year", "month", "day", "major", "minor", "patch")
        } else {
            ("major", "minor", "patch", "year", "month", "day")
        };

    set_i64(base, major_key, i64::from(version.base.major), "base", path)?;
    base.remove(old_major);

    match version.base.minor {
        Some(minor) => set_i64(base, minor_key, i64::from(minor), "base", path)?,
        None => {
            base.remove(minor_key);
        }
    }
    base.remove(old_minor);

    match version.base.patch {
        Some(patch) => set_i64(base, patch_key, i64::from(patch), "base", path)?,
        None => {
            base.remove(patch_key);
        }
    }
    base.remove(old_patch);

    Ok(())
}

fn write_version_into_doc(
    doc: &mut DocumentMut,
    version: &Version,
    path: &Path,
) -> Result<(), BumpError> {
    set_top_str(doc, "prefix", &version.prefix, path)?;

    let timestamp = table_mut(doc, "timestamp", path)?;
    set_str(
        timestamp,
        "format",
        &version.timestamp.format,
        "timestamp",
        path,
    )?;
    set_str(
        timestamp,
        "last",
        &version.timestamp.last,
        "timestamp",
        path,
    )?;

    write_base(doc, version, path)?;

    let phase = table_mut(doc, "phase", path)?;
    set_str(phase, "separator", &version.phase.separator, "phase", path)?;
    set_str(phase, "name", &version.phase.name, "phase", path)?;
    set_str(phase, "delimiter", &version.phase.delimiter, "phase", path)?;
    set_i64(
        phase,
        "distance",
        i64::from(version.phase.distance),
        "phase",
        path,
    )?;

    let suffix = table_mut(doc, "suffix", path)?;
    set_str(suffix, "mode", version.suffix.mode.as_str(), "suffix", path)?;
    set_str(
        suffix,
        "separator",
        &version.suffix.separator,
        "suffix",
        path,
    )?;

    let label = table_mut(doc, "label", path)?;
    set_str(
        label,
        "position",
        version.label.position.as_str(),
        "label",
        path,
    )?;

    Ok(())
}

impl BumpFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, BumpError> {
        let (path, doc) = read_doc(path.as_ref())?;
        Ok(Self { path, doc })
    }

    pub fn create(path: impl AsRef<Path>) -> Result<Self, BumpError> {
        let path = path.as_ref();
        ensure_directory_exists(path)?;

        let template_version: Version = {
            let content =
                include_str!("templates/bump.toml").replace("{timestamp}", INIT_TEMPLATE_TIMESTAMP);
            toml::from_str(&content).expect("init template must deserialize")
        };
        let current_timestamp = chrono::Utc::now()
            .format(&template_version.timestamp.format)
            .to_string();
        let content =
            include_str!("templates/bump.toml").replace("{timestamp}", &current_timestamp);

        fs::write(path, &content).map_err(BumpError::IoError)?;
        let doc = content
            .parse::<DocumentMut>()
            .map_err(|e| BumpError::ParseError(format!("Failed to parse TOML document: {e}")))?;

        Ok(Self {
            path: path.to_path_buf(),
            doc,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn version(&self) -> Result<Version, BumpError> {
        toml::from_str(&self.doc.to_string()).map_err(|err| {
            BumpError::ParseError(format!(
                "Failed to parse version from '{}': {err}. \
                Recreate your bumpfile with 'bump init'.",
                self.path.display()
            ))
        })
    }

    pub fn save(&mut self, version: &Version) -> Result<(), BumpError> {
        write_version_into_doc(&mut self.doc, version, &self.path)?;
        fs::write(&self.path, self.doc.to_string()).map_err(BumpError::IoError)
    }
}

impl TryFrom<&Path> for BumpFile {
    type Error = BumpError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        Self::load(path)
    }
}

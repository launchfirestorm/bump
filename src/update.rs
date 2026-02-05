
use crate::{bump::{BumpError, resolve_path}, version::Version};
use clap::ArgMatches;
use toml_edit::DocumentMut;
use std::fs;
use std::path::Path;


pub(crate) fn cargo_toml(
    version: &Version,
    path: &Path,
    repo_path: Option<&Path>,
) -> Result<(), BumpError> {
    // Read existing Cargo.toml
    let content = fs::read_to_string(path).map_err(BumpError::IoError)?;

    // Parse with toml_edit to preserve formatting
    let mut doc = content
        .parse::<DocumentMut>()
        .map_err(|e| BumpError::ParseError(format!("failed to parse {}: {}", path.display(), e)))?;

    // Update version in [package] section
    // Strip the prefix (e.g., "v") from the version string for Cargo.toml
    let version_str = version.fully_qualified_string(repo_path)?;
    let cargo_version = version_str
        .strip_prefix('v')
        .unwrap_or(&version_str);

    if let Some(package) = doc.get_mut("package") {
        package["version"] = toml_edit::value(cargo_version);
    } else {
        return Err(BumpError::ParseError(
            format!("no [package] section found in {}", path.display()),
        ));
    }

    fs::write(path, doc.to_string()).map_err(BumpError::IoError)?;
    println!("Cargo.toml updated to version {}", cargo_version);
    Ok(())
}

/// Update a file with the version from the bumpfile
pub fn modify_file(matches: &ArgMatches) -> Result<(), BumpError> {
    let version = Version::from_argmatches(matches)?;
    let path_str = matches
        .get_one::<String>("path")
        .ok_or_else(|| BumpError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path not provided"
        )))?;
    let file_path = resolve_path(path_str);
    
    match path_str.as_str() {
        "Cargo.toml" => cargo_toml(&version, &file_path, None),
        _ => Err(BumpError::LogicError(format!(
            "Unsupported file type: {}", path_str
        ))),
    }
}
use crate::{bump::{BumpError, resolve_path}, version::Version};
use clap::ArgMatches;
use toml_edit::DocumentMut;
use std::fs;
use std::path::Path;

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
        "Cargo.toml" => cargo_toml(&version, &file_path),
        "pyproject.toml" => pyproject_toml(&version, &file_path),
        _ => Err(BumpError::LogicError(format!(
            "Unsupported file type: {}", path_str
        ))),
    }
}

// NOTE: version.to_root_string() is preffered since we haven't tagged yet
// Cargo.toml doesn't accept a character prefix in the version number.
pub fn cargo_toml(
    version: &Version,
    path: &Path,
) -> Result<(), BumpError> {
    let content = fs::read_to_string(path).map_err(BumpError::IoError)?;

    let mut doc = content
        .parse::<DocumentMut>()
        .map_err(|e| BumpError::ParseError(format!("failed to parse {}: {}", path.display(), e)))?;

    // Cargo `package.version` must be semver without a leading `v` (or other prefix).
    let v_str = version.to_root_string(false)?;
    println!("cargo doesn't like a character prefix in Cargo.toml, stripping prefix");

    if let Some(package) = doc.get_mut("package") {
        package["version"] = toml_edit::value(v_str.as_str());
    } else {
        return Err(BumpError::ParseError(
            format!("no [package] section found in {}", path.display()),
        ));
    }

    fs::write(path, doc.to_string()).map_err(BumpError::IoError)?;
    println!("Cargo.toml updated to version {}", v_str);
    Ok(())
}

pub fn pyproject_toml(
    version: &Version,
    path: &Path,
) -> Result<(), BumpError> {
    let content = fs::read_to_string(path).map_err(BumpError::IoError)?;

    let mut doc = content .parse::<DocumentMut>()
        .map_err(|e| BumpError::ParseError(format!("failed to parse {}: {}", path.display(), e)))?;

    // https://packaging.python.org/en/latest/version.html#public-version-identifiers
    let yellow = "\x1b[33m";
    let cyan = "\x1b[36m";
    let purple = "\x1b[35m";
    let reset = "\x1b[0m";
    println!("{yellow}Warning: pyproject.toml version string must comply with the following scheme:{reset}");
    println!("{purple} [N!]N(.N)*[{{a|b|rc}}N][.postN][.devN]{reset}");
    println!("{cyan}  N, N!, and N.N are numeric components.{reset}");
    println!("{cyan}  {{a|b|rc}} is the alpha, beta, or release candidate suffix.{reset}");
    println!("{cyan}  postN is the post-release version.{reset}");
    println!("{cyan}  devN is the development version.{reset}");
    println!("{yellow}  Public version identifiers MUST NOT include leading or trailing whitespace.{reset}");

    let v_str = version.to_root_string(false)?;
    if let Some(project) = doc.get_mut("project") {
        project["version"] = toml_edit::value(v_str.as_str());
    }

    fs::write(path, doc.to_string()).map_err(BumpError::IoError)?;
    println!("pyproject.toml updated to version {}", v_str);
    Ok(())
}
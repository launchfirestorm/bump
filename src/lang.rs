use crate::bump::{BumpError, PrintType};
use crate::version::Version;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum Language {
    C,
    Go,
    Java,
    CSharp,
    Python,
}

impl Language {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "c" => Some(Self::C),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "csharp" => Some(Self::CSharp),
            "python" => Some(Self::Python),
            _ => None,
        }
    }

    const fn file_description(self) -> &'static str {
        match self {
            Self::C => "C header file",
            Self::Go => "Go source file",
            Self::Java => "Java source file",
            Self::CSharp => "C# source file",
            Self::Python => "Python source file",
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::CSharp => "C#",
            Self::Python => "Python",
        }
    }

    fn template(self, mode: &str) -> Result<&'static str, BumpError> {
        Ok(match (self, mode) {
            (Self::C, "semver") => include_str!("gen_tmpls/c/semver.h"),
            (Self::C, "calver") => include_str!("gen_tmpls/c/calver.h"),
            (Self::Go, "semver") => include_str!("gen_tmpls/go/semver.go"),
            (Self::Go, "calver") => include_str!("gen_tmpls/go/calver.go"),
            (Self::Java, "semver") => include_str!("gen_tmpls/java/semver.java"),
            (Self::Java, "calver") => include_str!("gen_tmpls/java/calver.java"),
            (Self::CSharp, "semver") => include_str!("gen_tmpls/csharp/semver.cs"),
            (Self::CSharp, "calver") => include_str!("gen_tmpls/csharp/calver.cs"),
            (Self::Python, "semver") => include_str!("gen_tmpls/python/semver.py"),
            (Self::Python, "calver") => include_str!("gen_tmpls/python/calver.py"),
            (_, mode) => return Err(unsupported_mode(self.name(), mode)),
        })
    }
}

struct OutputFields {
    version_string: String,
    timestamp: String,
    prefix: String,
    major: u32,
    minor: u32,
    patch: u32,
    phase: String,
}

fn output_fields(version: &Version) -> Result<OutputFields, BumpError> {
    Ok(OutputFields {
        version_string: version.to_string(&PrintType::Regular)?,
        timestamp: version.timestamp.last.clone(),
        prefix: version.version.prefix.clone(),
        major: version.version.major,
        minor: version.version.minor.unwrap_or(0),
        patch: version.version.patch.unwrap_or(0),
        phase: version.phase.name.clone(),
    })
}

fn unsupported_mode(lang: &str, mode: &str) -> BumpError {
    BumpError::LogicError(format!(
        "Unsupported version type for {lang} output: {mode}"
    ))
}

fn render_calver(tmpl: &str, f: &OutputFields) -> String {
    tmpl.replace("{version_string}", &f.version_string)
        .replace("{timestamp}", &f.timestamp)
}

fn render_semver(tmpl: &str, f: &OutputFields) -> String {
    tmpl.replace("{prefix}", &f.prefix)
        .replace("{major}", &f.major.to_string())
        .replace("{minor}", &f.minor.to_string())
        .replace("{patch}", &f.patch.to_string())
        .replace("{phase}", &f.phase)
        .replace("{version_string}", &f.version_string)
        .replace("{timestamp}", &f.timestamp)
}

fn render(lang: Language, mode: &str, f: &OutputFields) -> Result<String, BumpError> {
    let tmpl = lang.template(mode)?;
    Ok(match mode {
        "calver" => render_calver(tmpl, f),
        "semver" => render_semver(tmpl, f),
        mode => return Err(unsupported_mode(lang.name(), mode)),
    })
}

fn write_output(lang: Language, path: &Path, content: String) -> Result<(), BumpError> {
    fs::write(path, content).map_err(BumpError::IoError)?;
    println!("{} written to {}", lang.file_description(), path.display());
    Ok(())
}

fn lang_output(lang: Language, version: &Version, path: &Path) -> Result<(), BumpError> {
    let f = output_fields(version)?;
    let content = render(lang, version.version.mode.as_str(), &f)?;
    write_output(lang, path, content)
}

pub fn output_file(lang: Language, version: &Version, path: &Path) -> Result<(), BumpError> {
    lang_output(lang, version, path)
}

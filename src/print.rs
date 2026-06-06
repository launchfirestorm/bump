use crate::bump::{
    BumpError, get_git_branch, get_git_commit_sha, is_git_repository, resolve_path,
};
use crate::version::{LabelPosition, SuffixMode, Version, VersionMode};
use clap::ArgMatches;

// pub enum PrintType {
//     OnlyPrefix,
//     OnlyPhase,
//     OnlyBase,
//     NoPrefix,
//     NoPhase,
//     Regular,
//     WithSuffix,
//     WithTimestamp,
//     WithLabel(String),
//     Full,
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintOnly {
    Prefix,
    Base,
    Phase,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrintOptions {
    // pub regular: bool,
    pub only_prefix: bool,
    pub only_phase: bool,
    pub only_base: bool,
    pub no_prefix: bool,
    pub no_phase: bool,
    pub with_suffix: bool,
    pub with_timestamp: bool,
    pub with_label: Option<String>,
    pub full: bool,
}

pub fn parse_options(matches: &ArgMatches) -> Result<PrintOptions, BumpError> {
    let opts = PrintOptions {
        only_prefix: matches.get_flag("only-prefix"),
        only_phase: matches.get_flag("only-phase"),
        only_base: matches.get_flag("only-base"),
        no_prefix: matches.get_flag("no-prefix"),
        no_phase: matches.get_flag("no-phase"),
        with_suffix: matches.get_flag("with-suffix"),
        with_timestamp: matches.get_flag("with-timestamp"),
        with_label: matches.get_one::<String>("with-label").cloned(),
        full: matches.get_flag("full"),
    };

    if opts.only_prefix || opts.only_phase || opts.only_base {
        if opts.only_prefix && opts.only_phase 
            || opts.only_prefix && opts.only_base 
            || opts.only_phase && opts.only_base {
            return Err(BumpError::ParseError("Only one type of --only* allowed".to_string()));
        }
    }
    Ok(opts)
}

pub fn run(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = matches.get_one::<String>("bumpfile").unwrap();
    let version = Version::from_file(&resolve_path(bumpfile))?;
    let opts = parse_options(matches);
    print!("{}", assemble(&version, &opts)?);
    Ok(())
}

pub fn assemble(version: &Version, opts: &PrintOptions) -> Result<String, BumpError> {
    let prefix = &version.base.prefix;
    let base = base(version);
    let phase = phase(version);
    let pos = version.label.position;
    let suffix = suffix(version);
    let timestamp = version.timestamp.last;

    let output_str = String::new();

    if opts.only_prefix {
        return Ok(format!("{prefix}"));
    }
    if opts.only_base {
        return Ok(format!("{base}"));
    }
    if opts.only_phase {
        return Ok(format!("{phase}"));
    }

    if opts.no_prefix {
        return Ok(format!("{base}{phase}"));
    }
    if opts.no_phase {
        return Ok(format!("{prefix}{base}"));
    }
    if opts.with_suffix {
        return Ok(format!("{prefix}{base}{phase}{suffix}"));
    }
    if opts.with_timestamp {
        return Ok(format!("{prefix}{base}{phase}{timestamp}"));
    }
    if opts.with_label {
        return Ok(format!("{prefix}{base}{phase}{label}"));
    }
    if opts.full {
        return Ok(format!("{prefix}{base}{phase}{suffix}{timestamp}"));
    }
    return Ok(format!("{prefix}{base}{phase}"));
}

pub fn format(version: &Version, opts: &PrintOptions) -> Result<String, BumpError> {
    let prefix = &version.base.prefix;
    let base = base(version);
    let phase = phase(version);
    let pos = version.label.position;

    match opts {
        PrintOptions::OnlyPrefix => Ok(format!("{prefix}")),
        PrintOptions::OnlyPhase => Ok(format!("{phase}")),
        PrintOptions::OnlyBase => Ok(format!("{base}")),
        PrintOptions::NoPrefix => Ok(format!("{base}{phase}")),
    }
}

pub fn to_string(version: &Version, print_type: &PrintType) -> Result<String, BumpError> {
    let prefix = &version.base.prefix;
    let base = base(version);
    let phase = phase(version);
    let pos = version.label.position;

    match print_type {
        PrintType::Regular => Ok(core(prefix, &base, &phase, None, pos)),
        PrintType::NoPrefix => Ok(core("", &base, &phase, None, pos)),
        PrintType::WithTimestamp => Ok(format!(
            "{}  {}",
            core(prefix, &base, &phase, None, pos),
            version.timestamp.last
        )),
    }
}

fn core(
    prefix: &str,
    base: &str,
    phase: &str,
    label: Option<&str>,
    pos: LabelPosition,
) -> String {
    match (label, pos) {
        (None, _) => format!("{prefix}{base}{phase}"),
        (Some(label), LabelPosition::BeforeBase) => format!("{prefix}{label}{base}{phase}"),
        (Some(label), LabelPosition::AfterBase | LabelPosition::BeforePhase) => {
            format!("{prefix}{base}{label}{phase}")
        }
        (Some(label), LabelPosition::AfterPhase) => format!("{prefix}{base}{phase}{label}"),
    }
}

fn format_component(version: &Version, n: u32) -> String {
    if version.base.mode == VersionMode::Calver {
        format!("{n:02}")
    } else {
        n.to_string()
    }
}

fn base(version: &Version) -> String {
    match (version.base.minor, version.base.patch) {
        (Some(minor), Some(patch)) => format!(
            "{}{}{}{}{}",
            version.base.major,
            version.base.delimiter,
            format_component(version, minor),
            version.base.delimiter,
            format_component(version, patch),
        ),
        (Some(minor), None) => format!(
            "{}{}{}",
            version.base.major,
            version.base.delimiter,
            format_component(version, minor),
        ),
        (None, Some(patch)) => format!(
            "{}{}{}",
            version.base.major,
            version.base.delimiter,
            format_component(version, patch),
        ),
        _ => version.base.major.to_string(),
    }
}

fn phase(version: &Version) -> String {
    if version.phase.name.is_empty() && version.phase.distance == 0 {
        String::new()
    } else if version.phase.name.is_empty() && version.phase.distance > 0 {
        format!("{}{}", version.phase.prefix, version.phase.distance)
    } else if version.phase.distance == 0 {
        format!("{}{}", version.phase.prefix, version.phase.name,)
    } else {
        format!(
            "{}{}{}{}",
            version.phase.prefix,
            version.phase.name,
            version.phase.delimiter,
            version.phase.distance
        )
    }
}

fn suffix(version: &Version) -> Result<String, BumpError> {
    if !is_git_repository() {
        return Err(BumpError::Git("Not a git repository".to_string()));
    }
    match version.suffix.mode {
        SuffixMode::GitSha => {
            let sha = get_git_commit_sha()?;
            Ok(format!("{}{}", version.suffix.delimiter, sha))
        }
        SuffixMode::Branch => {
            let branch = get_git_branch()?;
            Ok(format!("{}{}", version.suffix.delimiter, branch))
        }
    }
}


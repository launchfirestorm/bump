use crate::bump::{
    BumpError, get_git_branch, get_git_commit_sha, is_git_repository, resolve_path,
};
use crate::version::{LabelPosition, SuffixMode, Version, VersionMode};
use clap::ArgMatches;


#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrintOptions {
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

impl PrintOptions {
    pub fn parse(matches: &ArgMatches) -> Result<Self, BumpError> {
        let opts = Self {
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
}

#[allow(dead_code)]
pub enum PrintType {
    Cli(PrintOptions, Components),
    // only types are used for retrieval
    NoPrefix,
    NoPhase,
    Regular,
    WithSuffix,
    WithTimestamp,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Field {
    active: bool,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LabelField {
    active: bool,
    value: Option<String>,
    position: LabelPosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Components {
    prefix: Field,
    base: Field,
    phase: Field,
    suffix: Field,
    timestamp: Field,
    label: LabelField,
}

impl Components {
    pub fn from(version: &Version, opts: &PrintOptions) -> Result<Self, BumpError> {
        Ok(Self {
            prefix: Field { active: true, value: version.base.prefix.clone() },
            base: Field { active: true, value: base(version).clone() },
            phase: Field { active: true, value: phase(version).clone() },
            suffix: Field { active: false, value: suffix(version)? },
            timestamp: Field { active: false, value: version.timestamp.last.clone() },
            label: LabelField { active: false, value: opts.with_label.clone(), position: version.label.position },
        })
    }

    // this is made right after a new call
    // pub fn set_print_type(&mut self, print_type: PrintType) {
    //     match print_type {
    //         PrintType::Cli(_, _) => (), // no-op is this won't be called here
    //         PrintType::NoPrefix => self.prefix.active = false,
    //         PrintType::NoPhase => self.phase.active = false,
    //         PrintType::Regular => (),
    //         PrintType::WithSuffix => self.suffix.active = true,
    //         PrintType::WithTimestamp => self.timestamp.active = true,
    //         PrintType::Full => {
    //             self.prefix.active = true;
    //             self.base.active = true;
    //             self.phase.active = true;
    //             self.suffix.active = true;
    //             self.timestamp.active = true;
    //         }
    //     }
    // }

    fn collect(&self) -> String {
        let mut output = String::new();
        if self.label.active && self.label.position == LabelPosition::BeforeBase {
            output.push_str(&self.label.value.clone().unwrap_or_default());
        }
        if self.prefix.active {
            output.push_str(&self.prefix.value);
        }
        if self.base.active {
            output.push_str(&self.base.value);
        }
        if self.label.active && self.label.position == LabelPosition::AfterBase {
            output.push_str(&self.label.value.clone().unwrap_or_default());
        }
        if self.label.active && self.label.position == LabelPosition::BeforePhase {
            output.push_str(&self.label.value.clone().unwrap_or_default());
        }
        if self.phase.active {
            output.push_str(&self.phase.value);
        }
        if self.label.active && self.label.position == LabelPosition::AfterPhase {
            output.push_str(&self.label.value.clone().unwrap_or_default());
        }
        if self.suffix.active {
            output.push_str(&self.suffix.value);
        }
        if self.timestamp.active {
            output.push_str(&self.timestamp.value);
        }
        output
    }
}


pub fn run(matches: &ArgMatches) -> Result<(), BumpError> {
    let bumpfile = matches.get_one::<String>("bumpfile").unwrap();
    let version = Version::from_file(&resolve_path(bumpfile))?;
    let opts = PrintOptions::parse(matches)?;
    let components = Components::from(&version, &opts)?;
    let print_type = PrintType::Cli(opts, components);
    print!("{}", to_string(&version, print_type)?);
    Ok(())
}

pub fn to_string(version: &Version, print_type: PrintType) -> Result<String, BumpError> {
    match print_type {
        PrintType::Cli(opts, mut components) => Ok(assemble(&opts, &mut components)?),
        PrintType::NoPrefix => Ok(format!("{}{}", base(version), phase(version))),
        PrintType::NoPhase => Ok(format!("{}{}", version.base.prefix, base(version))),
        PrintType::Regular => Ok(format!("{}{}{}", version.base.prefix, base(version), phase(version))),
        PrintType::WithSuffix => Ok(format!("{}{}{}{}", version.base.prefix, base(version), phase(version), suffix(version)?)),
        PrintType::WithTimestamp => Ok(format!("{}{}{}  {}", version.base.prefix, base(version), phase(version), version.timestamp.last)),
        PrintType::Full => Ok(format!("{}{}{}{}  {}", version.base.prefix, base(version), phase(version), suffix(version)?, version.timestamp.last)),
    }
}

pub fn assemble(opts: &PrintOptions, components: &mut Components) -> Result<String, BumpError> {
    // only options first
    if opts.only_prefix {
        return Ok(components.prefix.value.clone());
    }
    if opts.only_base {
        return Ok(components.base.value.clone());
    }
    if opts.only_phase {
        return Ok(components.phase.value.clone());
    }

    if opts.no_prefix {
        components.prefix.active = false;
    }
    if opts.no_phase {
        components.phase.active = false;
    }
    if opts.with_suffix {
        components.suffix.active = true;
    }
    if opts.with_timestamp {
        components.timestamp.active = true;
    }
    if opts.with_label.is_some() {
        components.label.active = true;
    }

    if opts.full {
        components.prefix.active = true;
        components.base.active = true;
        components.phase.active = true;
        components.suffix.active = true;
        components.timestamp.active = true;
    }
    Ok(components.collect())
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
        _ => format!("{}", version.base.major),
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


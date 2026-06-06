use clap::{Arg, Command};

#[allow(clippy::too_many_lines)]
pub fn cli() -> Command {
    Command::new("bump")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Automatic un-opinionated version bumping")
        .arg(
            Arg::new("bumpfile")
                .value_name("BUMPFILE")
                .value_parser(clap::value_parser!(String))
                .default_value("bump.toml")
                .global(true)
                .help("Path to the configuration file")
        )
        .subcommand(
            Command::new("init").about("Initialize a new version file with default values")
        )
        .subcommand(
            Command::new("gen")
                .about("Generate header files using git tag detection")
                .arg(
                    Arg::new("lang")
                        .short('l')
                        .long("lang")
                        .value_name("LANG")
                        .value_parser(clap::builder::PossibleValuesParser::new(["c", "java", "csharp", "go", "python"]))
                        .num_args(1)
                        .required(true)
                        .help("Programming language for output files")
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("OUTPUT")
                        .value_parser(clap::value_parser!(String))
                        .action(clap::ArgAction::Append)
                        .required(true)
                        .help("Output files for header generation (multiple files can be generated from a single bumpfile)")
                )
        )
        .subcommand(
            Command::new("tag")
                .about("Create a conventional git tag based on the current bumpfile version")
                .arg(
                    Arg::new("message")
                        .short('m')
                        .long("message")
                        .value_name("MESSAGE")
                        .value_parser(clap::value_parser!(String))
                        .help("Custom tag message (defaults to conventional commit format)")
                )
        )
        .subcommand(
            Command::new("update")
                .about("bump can update version in known file types (i.e: Cargo.toml)")
                .arg(
                    Arg::new("path")
                        .value_name("PATH")
                        .num_args(1)
                        .value_parser(clap::builder::PossibleValuesParser::new(["Cargo.toml", "pyproject.toml"]))
                        .required(true)
                        .help("Certain file types bump is aware of, and know how to update")
                )

        )
        .subcommand(Command::new("print")
            .about("Print [prefix][base][phase] from BUMPFILE without newline")
            .alias("p")
            .arg(
                Arg::new("only-prefix")
                    .long("only-prefix")
                    .action(clap::ArgAction::SetTrue)
                    .group("print-exclusive")
                    .help("Print [prefix]"),
            )
            .arg(
                Arg::new("only-phase")
                    .long("only-phase")
                    .action(clap::ArgAction::SetTrue)
                    .group("print-exclusive")
                    .help("Print [phase]"),
            )
            .arg(
                Arg::new("only-base")
                    .long("only-base")
                    .action(clap::ArgAction::SetTrue)
                    .group("print-exclusive")
                    .help("Print [base]"),
            )
            .arg(
                Arg::new("no-prefix")
                    .long("no-prefix")
                    .action(clap::ArgAction::SetTrue)
                    .group("print-stackable")
                    .help("Print [base][phase]"),
            )
            .arg(
                Arg::new("no-phase")
                    .long("no-phase")
                    .action(clap::ArgAction::SetTrue)
                    .group("print-stackable")
                    .help("Print [prefix][base]"),
            )
            .arg(
                Arg::new("with-suffix")
                    .long("with-suffix")
                    .action(clap::ArgAction::SetTrue)
                    .group("print-stackable")
                    .help("Print [prefix][base][phase][suffix]"),
            )
            .arg(
                Arg::new("with-timestamp")
                    .long("with-timestamp")
                    .action(clap::ArgAction::SetTrue)
                    .group("print-stackable")
                    .help("Print [prefix][base][phase][timestamp]"),
            )
            .arg(
                Arg::new("full")
                    .long("full")
                    .action(clap::ArgAction::SetTrue)
                    .group("print-stackable")
                    .help("Print [prefix][base][phase][suffix][timestamp]"),
            )
        )
        .arg(
            Arg::new("prefix")
                .long("prefix")
                .value_name("PREFIX")
                .value_parser(clap::value_parser!(String))
                .num_args(1)
                .help("Set prefix string (i.e: 'v', 'release-')")
        )
        .arg(
            Arg::new("suffix")
                .long("suffix")
                .value_name("MODE")
                .value_parser(clap::value_parser!(String))
                .num_args(1)
                .help("Set suffix mode 'git_sha' or 'branch'")
        )
        .arg(
            Arg::new("major")
                .long("major")
                .action(clap::ArgAction::SetTrue)
                .group("formal")
                .help("Increment major version"),
        )
        .arg(
            Arg::new("minor")
                .long("minor")
                .action(clap::ArgAction::SetTrue)
                .group("formal")
                .help("Increment minor version"),
        )
        .arg(
            Arg::new("patch")
                .long("patch")
                .action(clap::ArgAction::SetTrue)
                .group("formal")
                .help("Increment patch version"),
        )
        .arg(
            Arg::new("phase")
                .long("phase")
                .value_name("PHASE")
                .value_parser(clap::value_parser!(String))
                .num_args(0..=1)
                .default_missing_value("__increment__") // hidden from help
                .allow_hyphen_values(true)
                .group("formal")
                .help("Increment phase, if PHASE provided sets the phase name and resets distance.")
        )
        .arg(
            Arg::new("calendar")
                .long("calendar")
                .action(clap::ArgAction::SetTrue)
                .help("Update version based on current calendar date")
                .group("formal")
        )
}

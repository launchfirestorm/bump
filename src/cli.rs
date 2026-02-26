use clap::{Arg, Command};

pub fn cli() -> Command {
    Command::new("bump")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Automatic version bumping with sane defaults")
        .arg(
            Arg::new("bumpfile")
                .short('f')
                .long("file")
                .value_name("BUMPFILE")
                .value_parser(clap::value_parser!(String))
                .default_value("bump.toml")
                .global(true)
                .help("Path to the bumpfile to read version from")
        )
        .subcommand(
            Command::new("init")
                .about("Initialize a new version file with default values")
                .arg(
                    Arg::new("prefix")
                        .long("prefix")
                        .value_name("PREFIX")
                        .value_parser(clap::value_parser!(String))
                        .default_value("v")
                        .help("Prefix for version tags (e.g., 'v', 'release-', or empty string)")
                )
                .arg(
                    Arg::new("calver")
                        .long("calver")
                        .action(clap::ArgAction::SetTrue)
                        .help("Initialize with Calendar Versioning instead of Semantic Versioning")
                )
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
                        .value_name("output")
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
                        .value_parser(clap::builder::PossibleValuesParser::new(["Cargo.toml"]))
                        .required(true)
                        .help("Certain file types bump is aware of, and know how to update")
                )

        )
        .arg(
            Arg::new("print")
                .short('p')
                .long("print")
                .action(clap::ArgAction::SetTrue)
                .group("print-group")
                .help("Print version from BUMPFILE, without a newline"),
        )
        .arg(
            Arg::new("print-base")
                .short('b')
                .long("print-base")
                .action(clap::ArgAction::SetTrue)
                .group("print-group")
                .help("Print base version (no candidate suffix) from BUMPFILE, without a newline"),
        )
        .arg(
            Arg::new("print-with-timestamp")
                .long("print-with-timestamp")
                .action(clap::ArgAction::SetTrue)
                .group("print-group")
                .help("Print version with timestamp from BUMPFILE, without a newline"),
        )
        .arg(
            Arg::new("prefix")
                .long("prefix")
                .value_name("PREFIX")
                .value_parser(clap::value_parser!(String))
                .num_args(1)
                .group("meta")
                .help("Prefix for version tags (e.g., 'v', 'release-', or empty string)")
        )
        .arg(
            Arg::new("major")
                .long("major")
                .action(clap::ArgAction::SetTrue)
                .group("point-release")
                .conflicts_with_all(["meta", "candidate-release", "print-group"])
                .help("Bump the major version"),
        )
        .arg(
            Arg::new("minor")
                .long("minor")
                .action(clap::ArgAction::SetTrue)
                .group("point-release")
                .conflicts_with_all(["meta", "candidate-release", "print-group"])
                .help("Bump the minor version"),
        )
        .arg(
            Arg::new("patch")
                .long("patch")
                .action(clap::ArgAction::SetTrue)
                .group("point-release")
                .conflicts_with_all(["meta", "candidate-release", "print-group"])
                .help("Bump the patch version"),
        )
        .arg(
            Arg::new("release")
                .long("release")
                .action(clap::ArgAction::SetTrue)
                .group("point-release")
                .conflicts_with_all(["meta", "candidate-release", "print-group"])
                .help("Drop candidacy and promote to release")
        )
        .arg(
            Arg::new("candidate")
                .long("candidate")
                .action(clap::ArgAction::SetTrue)
                .help("if in candidacy increments the candidate version, otherwise creates a new candidate")
                .group("candidate-release")
                .conflicts_with_all(["point-release", "print-group"])
        )
}

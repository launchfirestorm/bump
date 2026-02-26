use crate::bump::BumpError;
use crate::lang::Language;
use std::process::ExitCode;

mod bump;
mod cli;
mod lang;
mod update;
#[cfg(test)]
mod tests;
mod version;

fn egress(result: Result<(), BumpError>) -> ExitCode {
    if let Err(err) = result {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let matches = cli::cli().get_matches();
    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            let bumpfile = sub_matches.get_one::<String>("bumpfile").unwrap();
            let prefix = sub_matches.get_one::<String>("prefix").unwrap();
            let use_calver = sub_matches.get_flag("calver");
            egress(bump::initialize(bumpfile, prefix, use_calver))
        }
        Some(("gen", sub_matches)) => {
            let lang_str = sub_matches
                .get_one::<String>("lang")
                .expect("LANG not provided");
            let lang = match Language::from_str(lang_str) {
                Some(l) => l,
                None => {
                    return egress(Err(BumpError::LogicError(format!(
                        "Invalid language specified: {lang_str}"
                    ))));
                }
            };
            egress(bump::generate(sub_matches, &lang))
        }
        Some(("tag", sub_matches)) => egress(bump::tag_version(sub_matches)),
        Some(("update", sub_matches)) => { egress(update::modify_file(sub_matches)) }
        _ => {
            if matches.contains_id("print-group") {
                let version = match bump::get_version(&matches) {
                    Ok(v) => v,
                    Err(err) => {
                        return egress(Err(err));
                    }
                };
                if matches.get_flag("print-with-timestamp") {
                    bump::print_with_timestamp(&version);
                } else {
                    bump::print(&version, matches.get_flag("print-base"));
                }
                ExitCode::SUCCESS
            } else if matches.contains_id("point-release")
                || matches.contains_id("candidate-release")
                || matches.get_one::<String>("prefix").is_some()
            {
                egress(bump::apply(&matches))
            } else {
                egress(Err(BumpError::LogicError(
                    "no action specified. Run with --help to see available options.".to_string(),
                )))
            }
        }
    }
}

use crate::bump::BumpError;
use crate::lang::Language;
use std::process::ExitCode;

mod bump;
mod cli;
mod lang;
#[cfg(test)]
mod tests;
mod update;
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
        Some(("init", sub_matches)) => egress(bump::initialize(sub_matches)),
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
        Some(("update", sub_matches)) => egress(update::modify_file(sub_matches)),
        Some(("print", sub_matches)) => egress(bump::print(sub_matches)),
        _ => {
            if matches.contains_id("meta") {
                egress(bump::meta(&matches))
            } else if matches.contains_id("formal") {
                egress(bump::apply(&matches))
            } else {
                egress(Err(BumpError::LogicError(
                    "No valid command specified".to_string()
                )))
            }
        }
    }
}

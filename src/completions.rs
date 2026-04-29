use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io;
 
use crate::Cli;
 
pub fn print_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
}

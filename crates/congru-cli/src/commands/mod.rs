mod align;
mod describe;
mod track;

use std::io::Write;

use serde::Serialize;

use crate::cli::Command;
use crate::methods::Method;
use crate::report::emit;

/// # Errors
/// When the command fails as a whole; a frame the tracker loses is a line
/// of its own, not a failure.
pub fn run(command: &Command, output: &mut dyn Write) -> anyhow::Result<()> {
    match command {
        Command::Align(arguments) => emit(output, &align::run(arguments)?),
        Command::Track(arguments) => track::run(arguments, output),
        Command::Describe(arguments) => emit(output, &describe::run(&arguments.file)?),
        Command::Methods => emit(output, &methods()),
    }
}

#[derive(Debug, Serialize)]
struct Listing {
    methods: Vec<Entry>,
}

#[derive(Debug, Serialize)]
struct Entry {
    name: &'static str,
    summary: &'static str,
    uses_guess: bool,
}

fn methods() -> Listing {
    Listing {
        methods: Method::ALL
            .iter()
            .map(|method| Entry {
                name: method.name(),
                summary: method.summary(),
                uses_guess: method.uses_guess(),
            })
            .collect(),
    }
}

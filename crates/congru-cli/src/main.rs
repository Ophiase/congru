//! The `congru` command: align a scan onto a model, follow a model through
//! a folder of frames, describe a point file. Every answer is JSON on
//! stdout, for a script or an agent to read; a failure is one
//! `{"error": "..."}` object and exit code 1.

mod cli;
mod commands;
mod files;
mod methods;
mod model;
mod report;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) if !error.use_stderr() => {
            return if error.print().is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        Err(error) => return fail(&usage(&error)),
    };
    match commands::run(&cli.command, &mut std::io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(&format!("{error:#}")),
    }
}

/// The first line of clap's message, which names the mistake; the usage
/// below it is for a person, who can ask `--help`.
fn usage(error: &clap::Error) -> String {
    let message = error.to_string();
    let first = message.lines().next().unwrap_or_default();
    first.strip_prefix("error: ").unwrap_or(first).to_owned()
}

fn fail(message: &str) -> ExitCode {
    let mut output = std::io::stdout().lock();
    let written = report::emit(&mut output, &serde_json::json!({ "error": message }));
    drop(output);
    if written.is_err() {
        return ExitCode::from(2);
    }
    ExitCode::FAILURE
}

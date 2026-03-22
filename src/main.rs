use std::env;
use std::io::{self, Read};
use std::process::ExitCode;

use waytrim::{Mode, render_preview, repair};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut mode = None;
    let mut preview = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "prose" => mode = Some(Mode::Prose),
            "command" => mode = Some(Mode::Command),
            "auto" => mode = Some(Mode::Auto),
            "--preview" => preview = true,
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    let mode = mode.unwrap_or(Mode::Prose);
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;

    let result = repair(&input, mode);

    if preview {
        print!("{}", render_preview(&input, &result));
        return Ok(());
    }

    print!("{}", result.output);
    Ok(())
}

fn print_help() {
    println!("waytrim <prose|command|auto> [--preview]");
}

use std::env;
use std::io::{self, Read};
use std::process::ExitCode;

use waytrim::cli::CliConfig;
use waytrim::{render_preview, repair};

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
    let args: Vec<String> = env::args().skip(1).collect();

    if args.iter().any(|arg| matches!(arg.as_str(), "-h" | "--help")) {
        print_help();
        return Ok(());
    }

    let config = CliConfig::parse(args.iter().map(String::as_str))?;

    if config.clipboard {
        return Err(String::from("clipboard mode not implemented yet"));
    }

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;

    let result = repair(&input, config.mode);

    if config.preview {
        print!("{}", render_preview(&input, &result));
        return Ok(());
    }

    print!("{}", result.output);
    Ok(())
}

fn print_help() {
    println!("waytrim prose");
    println!("waytrim command");
    println!("waytrim auto");
    println!("waytrim prose --preview");
    println!("waytrim prose --clipboard");
    println!("waytrim prose --clipboard --print");
    println!("waytrim prose --clipboard --preview");
}

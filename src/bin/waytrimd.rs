use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use waytrim::{ServiceConfig, default_socket_path, run_service};

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
    let mut args = env::args().skip(1);
    let mut socket_path = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "--socket" => {
                let path = args
                    .next()
                    .ok_or_else(|| String::from("--socket requires a path"))?;
                socket_path = Some(PathBuf::from(path));
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    run_service(&ServiceConfig {
        socket_path: socket_path.unwrap_or_else(default_socket_path),
    })
}

fn print_help() {
    println!("waytrimd");
    println!("waytrimd --socket /path/to/waytrim.sock");
}

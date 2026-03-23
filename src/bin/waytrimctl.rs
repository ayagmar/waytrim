use std::env;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use waytrim::ipc::send_request;
use waytrim::{IpcRequest, IpcResponse, Mode, default_socket_path};

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
    let Some(command) = args.first().map(String::as_str) else {
        print_help();
        return Ok(());
    };

    match command {
        "-h" | "--help" => {
            print_help();
            Ok(())
        }
        "repair" => run_repair(&args[1..]),
        "shutdown" => run_shutdown(&args[1..]),
        other => Err(format!("unknown command: {other}")),
    }
}

fn run_repair(args: &[String]) -> Result<(), String> {
    let Some(mode) = args.first() else {
        return Err(String::from("repair requires a mode"));
    };

    let mode = parse_mode(mode)?;
    let mut socket_path = default_socket_path();
    let mut text_output = false;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--socket" => {
                index += 1;
                let path = args
                    .get(index)
                    .ok_or_else(|| String::from("--socket requires a path"))?;
                socket_path = PathBuf::from(path);
            }
            "--text" => text_output = true,
            "--json" => text_output = false,
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }

    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;

    let response = send_request(&socket_path, &IpcRequest::repair(mode, input))?;
    match response {
        IpcResponse::Ok { report, .. } if text_output => {
            print!("{}", report.output);
            Ok(())
        }
        IpcResponse::Ok { .. } => {
            print!(
                "{}",
                serde_json::to_string(&response)
                    .map_err(|error| format!("failed to encode response: {error}"))?
            );
            Ok(())
        }
        IpcResponse::Error { message, .. } => Err(message),
        IpcResponse::Ack { message, .. } => Err(format!("unexpected service ack: {message}")),
    }
}

fn run_shutdown(args: &[String]) -> Result<(), String> {
    let mut socket_path = default_socket_path();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--socket" => {
                index += 1;
                let path = args
                    .get(index)
                    .ok_or_else(|| String::from("--socket requires a path"))?;
                socket_path = PathBuf::from(path);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }

    let response = send_request(&socket_path, &IpcRequest::shutdown())?;
    match response {
        IpcResponse::Ack { message, .. } => {
            println!("{message}");
            Ok(())
        }
        IpcResponse::Error { message, .. } => Err(message),
        IpcResponse::Ok { .. } => Err(String::from("unexpected repair response")),
    }
}

fn parse_mode(value: &str) -> Result<Mode, String> {
    match value {
        "prose" => Ok(Mode::Prose),
        "command" => Ok(Mode::Command),
        "auto" => Ok(Mode::Auto),
        other => Err(format!("invalid mode: {other}")),
    }
}

fn print_help() {
    println!("waytrimctl repair prose");
    println!("waytrimctl repair command --socket /path/to/waytrim.sock");
    println!("waytrimctl repair auto --text");
    println!("waytrimctl shutdown");
}

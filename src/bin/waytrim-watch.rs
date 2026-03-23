use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use waytrim::clipboard::SystemClipboard;
use waytrim::config::load_user_defaults;
use waytrim::{
    AutoClipboardConfig, AutoClipboardStatus, Mode, WatchPaths, restore_last_original,
    run_auto_clipboard_once,
};

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
    let parsed = ParsedArgs::parse(args.iter().map(String::as_str))?;

    if parsed.help {
        print_help();
        return Ok(());
    }

    let (defaults, warning) = load_user_defaults();
    if let Some(warning) = warning {
        eprintln!("{warning}");
    }

    let config = AutoClipboardConfig {
        mode: parsed.mode.unwrap_or(Mode::Auto),
        policy: defaults.policy,
    };
    let state_path = parsed
        .state_path
        .clone()
        .unwrap_or_else(|| WatchPaths::default().state_path);
    let paths = WatchPaths { state_path };

    if parsed.restore_original {
        let output = restore_last_original(&SystemClipboard::new(), &paths)?;
        eprint!("{}", output.message);
        return Ok(());
    }

    if parsed.hook {
        let output = run_auto_clipboard_once(&config, &SystemClipboard::new(), &paths)?;
        if !matches!(output.status, AutoClipboardStatus::Skipped) {
            eprint!("{}", output.message);
        }
        return Ok(());
    }

    let current_exe = env::current_exe()
        .map_err(|error| format!("failed to locate current executable: {error}"))?;
    let mut command = Command::new("wl-paste");
    command.arg("--watch").arg(&current_exe).arg("--hook");

    if let Some(mode) = parsed.mode {
        command.arg(mode.as_str());
    }

    if let Some(path) = parsed.state_path {
        command.arg("--state-path").arg(path);
    }

    let status = command
        .status()
        .map_err(|error| format!("failed to start wl-paste watch mode: {error}"))?;

    if status.success() {
        return Ok(());
    }

    Err(format!("wl-paste watch mode exited with {status}"))
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ParsedArgs {
    mode: Option<Mode>,
    state_path: Option<PathBuf>,
    restore_original: bool,
    hook: bool,
    help: bool,
}

impl ParsedArgs {
    fn parse<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut parsed = Self::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_ref() {
                "prose" => parsed.mode = Some(Mode::Prose),
                "command" => parsed.mode = Some(Mode::Command),
                "auto" => parsed.mode = Some(Mode::Auto),
                "--restore-original" => parsed.restore_original = true,
                "--hook" => parsed.hook = true,
                "-h" | "--help" => parsed.help = true,
                "--state-path" => {
                    let Some(path) = args.next() else {
                        return Err(String::from("--state-path requires a path"));
                    };
                    parsed.state_path = Some(PathBuf::from(path.as_ref()));
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        Ok(parsed)
    }
}

fn print_help() {
    println!("waytrim-watch");
    println!("waytrim-watch auto");
    println!("waytrim-watch prose");
    println!("waytrim-watch --restore-original");
    println!("waytrim-watch --state-path /path/to/watch-state.json");
}

use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use waytrim::clipboard::SystemClipboard;
use waytrim::config::load_user_defaults;
use waytrim::{
    AutoClipboardConfig, AutoClipboardStatus, Mode, WatchPaths, read_watch_status,
    record_watch_error, restore_last_original, run_auto_clipboard_once, run_manual_clipboard_once,
    write_watch_idle_status,
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
    parsed.validate()?;

    if parsed.help {
        print_help();
        return Ok(());
    }

    let state_path = parsed
        .state_path
        .clone()
        .unwrap_or_else(|| WatchPaths::default().state_path);
    let paths = WatchPaths { state_path };

    if parsed.status {
        let status = read_watch_status(&paths)?;
        if parsed.json {
            println!(
                "{}",
                serde_json::to_string(&status)
                    .map_err(|error| format!("failed to encode watch status: {error}"))?
            );
        } else {
            print_status(&status);
        }
        return Ok(());
    }

    if parsed.restore_original {
        let output = restore_last_original(&SystemClipboard::new(), &paths)?;
        eprint!("{}", output.message);
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

    if parsed.clean_once {
        let output = run_manual_clipboard_once(&config, &SystemClipboard::new(), &paths)?;
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

    write_watch_idle_status(&paths, config.mode)?;

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

    let message = format!("wl-paste watch mode exited with {status}");
    let _ = record_watch_error(&paths, Some(config.mode), &message);
    Err(message)
}

fn print_status(status: &waytrim::WatchStatusSnapshot) {
    println!("status: {}", status.status.as_str());
    println!("message: {}", status.message);
    println!(
        "mode: {}",
        status.mode.map(Mode::as_str).unwrap_or("unknown")
    );
    println!("original_available: {}", status.original_available);
    println!("clipboard_source: {}", status.clipboard_source.as_str());
    println!("event_id: {}", status.event_id);
    match status.updated_at_ms {
        Some(updated_at_ms) => println!("updated_at_ms: {updated_at_ms}"),
        None => println!("updated_at_ms: unknown"),
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ParsedArgs {
    mode: Option<Mode>,
    state_path: Option<PathBuf>,
    restore_original: bool,
    clean_once: bool,
    status: bool,
    json: bool,
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
                "--clean-once" => parsed.clean_once = true,
                "--status" => parsed.status = true,
                "--json" => parsed.json = true,
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

    fn validate(&self) -> Result<(), String> {
        let actions = [
            self.restore_original,
            self.clean_once,
            self.status,
            self.hook,
        ]
        .into_iter()
        .filter(|enabled| *enabled)
        .count();

        if actions > 1 {
            return Err(String::from(
                "choose only one of --restore-original, --clean-once, --status, or --hook",
            ));
        }

        if self.json && !self.status {
            return Err(String::from("--json is only supported with --status"));
        }

        Ok(())
    }
}

fn print_help() {
    println!("waytrim-watch");
    println!("watch and conservatively repair Wayland clipboard updates");
    println!();
    println!("Usage:");
    println!("  waytrim-watch auto");
    println!("  waytrim-watch prose");
    println!("  waytrim-watch command");
    println!("  waytrim-watch --clean-once auto");
    println!("  waytrim-watch --restore-original");
    println!("  waytrim-watch --status");
    println!("  waytrim-watch --status --json");
    println!("  waytrim-watch --state-path /path/to/watch-state.json");
    println!();
    println!("Notes:");
    println!("  --clean-once ignores the self-update skip guard for one manual override");
    println!("  --restore-original restores the most recent saved pre-clean clipboard text");
    println!("  --status --json is the machine-readable status surface for desktop adapters");
}

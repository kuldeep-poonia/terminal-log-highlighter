mod runtime;
mod matcher;
mod renderer;
mod config;
mod parser;
mod terminal;

use std::env;
use terminal::Session;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // ── CLI flags 
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("sentinel {VERSION}");
        return;
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    // ── Load config 
    let config = config::load_config();

    // ── Run ──────
    //
    // Two operating modes:
    //
    //   1. FILTER MODE (no arguments):
    //        docker compose up --build | sentinel
    //      Sentinel reads from stdin and writes highlighted output to stdout.
    //
    //   2. SPAWN MODE (command + args given):
    //        sentinel docker compose up --build
    //      Sentinel spawns the command inside a PTY so the child thinks it
    //      has a real terminal.  This preserves colour output, interactive
    //      prompts, Ctrl+C handling, and window-size awareness.
    //
    let result = if args.is_empty() {
        let mut session = Session::stdin();
        runtime::run(session.reader(), config)
    } else {
        let command  = &args[0];
        let cmd_args = &args[1..];
        match Session::spawn(command, cmd_args) {
            Ok(mut session) => runtime::run(session.reader(), config),
            Err(e) => {
                eprintln!("sentinel: could not spawn '{}': {}", command, e);
                std::process::exit(1);
            }
        }
    };

    if let Err(e) = result {
        std::process::exit(e.raw_os_error().unwrap_or(1));
    }
}

fn print_help() {
    println!("sentinel {VERSION} — invisible terminal log highlighter");
    println!();
    println!("USAGE:");
    println!("  sentinel [OPTIONS] [COMMAND [ARGS...]]");
    println!();
    println!("MODES:");
    println!("  Pipe filter   docker compose up --build | sentinel");
    println!("  Spawn mode    sentinel docker compose up --build");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help       Print this help message");
    println!("  -V, --version    Print version number");
    println!();
    println!("CONFIGURATION:");
    println!("  Config is loaded from the first source found:");
    println!("    $SENTINEL_CONFIG   explicit path via environment variable");
    println!("    ./.sentinel.toml   project-local config");
    println!("    ~/.sentinel.toml   user-wide config");
    println!("    (built-in defaults if none of the above exist)");
    println!();
    println!("HIGHLIGHTS:");
    println!("  critical  bold yellow on red background  + terminal bell");
    println!("  error     bold white  on red background  + terminal bell");
    println!("  warn      bold yellow");
    println!("  info      bold cyan");
}
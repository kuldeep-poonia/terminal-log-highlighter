mod runtime;
mod matcher;
mod renderer;
mod config;
mod parser;
mod terminal;

use std::env;
use std::io::{self, BufRead};
use terminal::Session;

fn main() {
    let config = config::load_config();

    // Collect command‑line arguments (skip the binary name).
    let args: Vec<String> = env::args().skip(1).collect();

    let result = if args.is_empty() {
        // No arguments → read from stdin.
        let mut session = Session::stdin();
        runtime::run(session.reader(), config)
    } else {
        // Spawn the command with PTY.
        let command = &args[0];
        let cmd_args = &args[1..];
        match Session::spawn(command, cmd_args) {
            Ok(mut session) => runtime::run(session.reader(), config),
            Err(e) => {
                // Invisible runtime: we could log to a hidden file in the future.
                eprintln!("sentinel: could not spawn command: {}", e);
                std::process::exit(1);
            }
        }
    };

    if let Err(e) = result {
        std::process::exit(e.raw_os_error().unwrap_or(1));
    }
}
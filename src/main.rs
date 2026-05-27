mod runtime;
mod matcher;
mod renderer;
mod config;

fn main() {
    let config = config::load_config();
    if let Err(e) = runtime::run(config) {
        std::process::exit(e.raw_os_error().unwrap_or(1));
    }
}
mod runtime;
mod matcher;
mod renderer;

fn main() {
    // Invisible runtime: no output on failure.
    // In a later phase, SENTINEL_DEBUG will enable hidden diagnostics.
    if let Err(e) = runtime::run() {
        std::process::exit(e.raw_os_error().unwrap_or(1));
    }
}
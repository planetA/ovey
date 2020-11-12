//! This is a dummy target in order to allow Cargo to have a common workspace across all Ovey
//! userland sub projects. This has the advantage that there is only a single target folder.
//! Less disk usage and faster builds (because much is in common).
//!
//! The real contribution are the binaries from the sub projects (CLI, COORDINATOR, DAEMON).

fn main() {
    println!("I'm just a dummy target!")
}
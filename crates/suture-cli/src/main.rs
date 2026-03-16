use suture_fetch as _;
use suture_index as _;
use suture_manifest as _;
use suture_resolver as _;
use suture_verify as _;

pub mod commands;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if let Err(message) = commands::dispatch(&args) {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

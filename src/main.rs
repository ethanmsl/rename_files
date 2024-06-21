//! CLI interface to allow regex based file renaming

use clap::Parser;
#[derive(Parser, Debug)]
#[command(version, about)]
/// Struct info
struct Args {
    // like element to search for in subdomain names
    likeness: String,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world!");
}

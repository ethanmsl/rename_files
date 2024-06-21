//! CLI interface to allow regex based file renaming

// Notes:
// Show files that can and can't be written to.
// Show files that do and don't match patterns (?)
// Have limits on what's shown?

use clap::Parser;
use regex::Regex;
use rename_files::{error::Result, logging::tracing_subscribe_boilerplate};
use tracing::{error, info, warn};
use walkdir::WalkDir;

/// Struct info
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// like element to search for in subdomain names
    #[arg(short, long)]
    regex: String,

    /// indicate transformations that would occur
    #[arg(short, long)]
    test_run: bool,
    //
}

fn main() -> Result<()> {
    // Setup logging
    tracing_subscribe_boilerplate("info");
    tracing::info!("Starting up!");

    // Get Args
    let args = Args::parse();
    tracing::info!("Args: {:?}", args);
    // translate to regex
    let re = Regex::new(&args.regex)?;

    // Get Files
    for entry in WalkDir::new(".") {
        let Ok(entry) = entry else {
            tracing::error!("Entry that caused a walkdir error: {:?}", entry);
            continue;
        };
        let Some(entry) = entry.path().to_str() else {
            tracing::error!("Entry that caused a to_string error: {:?}", entry);
            continue;
        };
        let Some(captures) = re.captures(entry) else {
            tracing::debug!("No Match for Entry: {:?}", entry);
            continue;
        };
        tracing::info!("Entry: {:?}", entry);
        tracing::info!("The caps are: {:?}", &captures);
    }

    // Validate in Regex
    // Validate out Regex

    // Find Matching files
    // Find matching + Writeable files
    // (cache both separately)

    // Simulate actions
    // - check if valid paths
    // - check if 'reasonable' paths (e.g. length)

    // if --change-yes
    // serially change files, logging each and any errors
    Ok(())
}

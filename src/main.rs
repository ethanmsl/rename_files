//! CLI interface to allow regex based file renaming

// Notes:
// Show files that can and can't be written to.
// Show files that do and don't match patterns (?)
// Have limits on what's shown?

use clap::Parser;
use itertools::Itertools;
use regex::Regex;
use rename_files::{error::Result, logging::tracing_subscribe_boilerplate};
use tracing::{error, info, warn};
use walkdir::WalkDir;

/// CLI input arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// like element to search for in subdomain names
    #[arg(long)]
    regex: String,

    /// recurse into child directories
    #[arg(short, long)]
    recurse: bool,

    /// indicate transformations that would occur
    #[arg(short, long)]
    test_run: bool,
}

fn main() -> Result<()> {
    // Setup logging
    tracing_subscribe_boilerplate("info");
    tracing::info!("Starting up!");

    let boop = WalkDir::new(".").min_depth(1).into_iter().filter_entry(|e| !is_hidden(e));

    // Get Args
    let args = Args::parse();
    // translate to regex
    let re = Regex::new(&args.regex)?;

    // Recurse?
    let walkable_space = if args.recurse {
        tracing::debug!("Recursable WalkDir");
        WalkDir::new(".").min_depth(1)
    } else {
        tracing::debug!("unrecursable WalkDir");
        WalkDir::new(".").min_depth(1).max_depth(1)
    };

    // Check Files
    for entry in walkable_space {
        // Guard: walk errors (e.g. loop encountered)
        let Ok(entry) = entry else {
            tracing::error!("Error encounered while walking dir: {:?}", entry);
            continue;
        };
        // Guard: path->to->str errors (e.g. non-utf8 paths)
        let Some(entry) = entry.path().to_str() else {
            tracing::error!("Entry that caused a to_string error: {:?}", entry);
            continue;
        };
        // lightGuard: no regex match
        let Some(captures) = re.captures(entry) else {
            tracing::debug!("No Match for Entry: {:?}", entry);
            continue;
        };

        println!("for regex '{}' a match was found in entry {}", &args.regex, &entry);
        println!("The captures are: {:?}", &captures);
    }

    // if --change-yes
    // serially change files, logging each and any errors
    Ok(())
}

/// Checks for a literal `.` prefix on a entry
///
/// # Note: This will trigger on the `.` used to indicate the 'local' directory
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    let is_hidden = entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false);
    if is_hidden {
        tracing::trace!("Ignoring hidden file: {:?}", entry.path());
    } else {
        tracing::trace!("Not a hidden file: {:?}", entry.path());
    }
    is_hidden
}

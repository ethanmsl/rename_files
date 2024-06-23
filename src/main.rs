//! CLI interface to allow regex based file renaming
//!
//! # Example:
//! ```bash
//! clear; el; carr -- --regex '(C|c)argo.*(\..*)' --replacement '$1ogra$2'
//! ```

// Notes:
// Show files that can and can't be written to.
// Show files that do and don't match patterns (?)
// Have limits on what's shown?

use clap::Parser;
use owo_colors::OwoColorize;
use regex::Regex;
use rename_files::{error::Result, logging::tracing_subscribe_boilerplate};
use walkdir::WalkDir;

// regex for checking a reference number followed by other chars
// e.g. `$1abc` will be parsed as ($1abc) NOT ($1)(abc)
//      `${1}abc` is proper syntax
const RE_SYNTAX_WARN: &str = r"(\$\d)[^\d\$\s]+";

/// CLI input arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// like element to search for in subdomain names
    regex: String,

    /// like element to search for in subdomain names
    #[arg(long = "rep")]
    replacement: Option<String>,

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
    tracing::trace!("Starting up!");

    // Get Args
    let args = Args::parse();
    // translate to regex
    let re = Regex::new(&args.regex)?;

    // Guard: Flagging unintended syntax
    if let Some(rep_arg) = &args.replacement {
        let re_check = Regex::new(RE_SYNTAX_WARN).expect("valid, static regex");
        if let Some(cap) = re_check.captures(rep_arg) {
            tracing::warn!("\nWarning:\ncapture reference `{}` is being read as `{}`\nIf this is not intended use: `${{_}}...` instead.",
                           cap[1].to_string().blue(),
                           cap[0].to_string().red());
            return Err("Ambiguous replacement syntax".into());
        }
    }

    // Set: Recurse?
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
            tracing::trace!("No Match for Entry: {:?}", entry);
            continue;
        };

        tracing::debug!("for regex '{}' a match was found in entry {}", &args.regex.green(), &entry.purple());
        tracing::debug!("The captures are: {:?}", &captures.blue());

        if let Some(rep) = &args.replacement {
            let new_name = re.replace(entry, rep);
            println!("Mapping {} ~~> {}", &entry.black().bold().on_green(), &new_name.red().bold().on_blue());

            if !args.test_run {
                std::fs::rename(entry, new_name.as_ref())?;
                println!("...file renamed\n");
            } else {
                println!("'--test-run' active, no files renamed\n");
            }
        } else {
            println!("Match found: {}", &entry.black().bold().on_green());
        }
    }

    Ok(())
}

/// Checks for a literal `.` prefix on a entry
///
/// # Note: This will trigger on the `.` used to indicate the 'local' directory
#[allow(unused)]
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    let is_hidden = entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false);
    if is_hidden {
        tracing::trace!("Ignoring hidden file: {:?}", entry.path());
    } else {
        tracing::trace!("Not a hidden file: {:?}", entry.path());
    }
    is_hidden
}

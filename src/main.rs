//! CLI interface to allow regex based file renaming
//!
//! # Example:
//! ```bash
//! clear; el; carr -- '(C|c)argo.*(\..*)' --rep '$1ogra$2' --test-run
//! clear; el; carr -- '(C|c)argo.*(\..*)' --rep '${1}ogra$2' --test-run
//! ```

// Notes:
// Show files that can and can't be written to.
// Show files that do and don't match patterns (?)
// Have limits on what's shown?

use clap::Parser;
use error::Result;
use logging::tracing_subscribe_boilerplate;
use owo_colors::OwoColorize;
use regex::Regex;
use walkdir::WalkDir;

/// Filename Find and (optionally) Replace using Rust Regex Syntax.  
///
/// Files are only renamed if a `--rep(lace)` argument is provided
/// AND `--test-run` or `-t` is not provided.  
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// (Rust flavor) regex to search filenames with.
    regex: String,

    /// Replacement string for regex matches. Use `$1`, `${1}` to reference capture groups.
    #[arg(long = "rep")]
    replacement: Option<String>,

    /// Recurse into child directories.
    #[arg(short, long)]
    recurse: bool,

    /// Show replacements that would occur, but don't rename files.
    #[arg(short, long)]
    test_run: bool,
}

fn main() -> Result<()> {
    tracing_subscribe_boilerplate("info");

    let args = Args::parse();
    let re = Regex::new(&args.regex)?;

    if let Some(replacement) = &args.replacement {
        check_for_common_syntax_error(replacement)?;
    }

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

/// Guard: Flagging unintended syntax
///
/// Checkes replacement string for capture references making a common syntax error:
/// A bare reference number followed by chars that would be combined with it and read as a name
///
/// e.g. `$1abc` will be parsed as ($1abc) NOT ($1)(abc) -- `${1}abc` is proper syntax
fn check_for_common_syntax_error(rep_arg: &str) -> Result<()> {
    const RE_SYNTAX_WARN: &str = r"(\$\d)[^\d\$\s]+";

    let re_check = Regex::new(RE_SYNTAX_WARN).expect("valid, static regex");
    if let Some(cap) = re_check.captures(rep_arg) {
        tracing::warn!("\nWarning:\ncapture reference `{}` is being read as `{}`\nIf this is not intended use: `${{_}}...` instead.",
                       cap[1].to_string().blue(),
                       cap[0].to_string().red());
        return Err("Ambiguous replacement syntax".into());
    }
    Ok(())
}

/// Very early error type that can flow into standard error handling with type coercions.
pub mod error {
    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;
}

/// Simple logging boilerplate code.
pub mod logging {
    use tracing_subscriber::EnvFilter;

    pub fn tracing_subscribe_boilerplate(env_min: impl Into<String>) {
        let filter = EnvFilter::try_new(
                    std::env::var("RUST_LOG").unwrap_or_else(|_| env_min.into()),
                )
                .expect("Valid filter input provided.");

        tracing_subscriber::fmt().pretty()
                                 .with_env_filter(filter)
                                 .with_file(true)
                                 .with_line_number(true)
                                 .with_thread_ids(true)
                                 .with_target(true)
                                 .init();
    }
}

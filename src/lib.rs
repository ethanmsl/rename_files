//! CLI interface to allow regex based file renaming

pub mod error;
pub mod logging;

use clap::Parser;
use error::Result;
use owo_colors::OwoColorize;
use regex::Regex;
use walkdir::WalkDir;

/// Filename Find and (optionally) Replace using Rust Regex Syntax.  
///
/// Files are only renamed if a `--rep(lace)` argument is provided AND `--test-run` or `-t` is *not* provided.  
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// (Rust flavor) regex to search filenames with.
    regex: String,

    /// Replacement string for regex matches. Use `$1` or `${1}`, etc. to reference capture groups.
    #[arg(long = "rep")]
    replacement: Option<String>,

    /// Recurse into child directories.
    #[arg(short, long)]
    recurse: bool,

    /// Show replacements that would occur, but don't rename files.
    #[arg(short, long)]
    test_run: bool,
}

/// Application code.  (main in lib.rs)
pub fn app() -> Result<()> {
    let args = Args::parse();
    let re = Regex::new(&args.regex)?;

    if let Some(replacement) = &args.replacement {
        check_for_common_syntax_error(replacement)?;
    }

    let walkable_space = walkdir_build_with_depths(args.recurse);

    let mut matches = 0;
    for entry in walkable_space.into_iter() {
        // Guard: walk errors (e.g. loop encountered)
        let Ok(entry) = entry else {
            tracing::error!("Error encountered while walking dir: {:?}", entry);
            continue;
        };
        // Guard: path->to->str errors (e.g. non-utf8 paths)
        let Some(entry) = entry.path().to_str() else {
            tracing::error!("Entry path could not convert to a string: {:?}", entry);
            continue;
        };
        // Guard: no regex match
        let Some(_) = re.find(entry) else {
            tracing::trace!("No Match for Entry: {:?}", entry);
            continue;
        };
        matches += 1;
        // Branch: no replacement
        let Some(rep) = &args.replacement else {
            println!("Match found: {}", &entry.black().bold().on_green());
            continue;
        };
        let new_name = re.replace(entry, rep);
        // Branch: Rename ||| --test-run
        if !args.test_run {
            println!("Renaming: {} ~~> {}", &entry.black().bold().on_green(), &new_name.red().bold().on_blue());
            std::fs::rename(entry, new_name.as_ref())?;
        } else {
            println!("--test-run mapping: {} ~~> {}",
                     &entry.black().bold().on_green(),
                     &new_name.red().bold().on_blue());
        }
    }
    println!("Total matches: {}", matches.cyan());

    Ok(())
}

/// Guard: Flagging unintended syntax
///
/// Checks replacement string for capture references making a common syntax error:
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

/// Build a WalkDir object with depth limits based information passed in
fn walkdir_build_with_depths(does_recurse: bool) -> WalkDir {
    if does_recurse {
        tracing::debug!("Recursable WalkDir");
        return WalkDir::new(".").min_depth(1);
    }

    tracing::debug!("non-recursing (shallow) WalkDir");
    WalkDir::new(".").min_depth(1).max_depth(1)
}

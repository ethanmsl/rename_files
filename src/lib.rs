//! CLI interface to allow regex based file searching and renaming
//! This is just designed for my personal needs and functionality and ergonomics only added as needed.

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
pub struct Args {
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
#[tracing::instrument]
pub fn app(args: &Args) -> Result<()> {
    let re = Regex::new(&args.regex)?;

    if let Some(replacement) = &args.replacement {
        check_for_common_syntax_error(replacement)?;
    }
    let walkable_space = walkdir_build_with_depths(args.recurse);
    core_process_loop(walkable_space, &re, args)
}

/// Walks a WalkDir, handles errors, prints matches, optionally executes
///
/// # Note 1, single-purpose violation:
/// Breaking this function into smaller pieces would create indirection and complexity
/// for very little benefit given the brevity of the code and the linear logic chain at work.
/// (this does come at the cost of a somewhat ambiguous function name :shrug:)
///
/// # Note 2, loop vs iterator choice:
/// Would be charming as an iterator.  Perhaps using itertools `map_ok` to transform
/// elements passing successive guards.  And then using raw error messages to generate logs.
/// Or `filter_map` with inspects to create similar behavior (and hope compiler notices the double checking of Result & Options).
/// BUT: while charming, the lack of shared scope makes passing references along past multiple
/// guards quite awkward.  And the workarounds end up being deeply nested and more verbose
/// without any clear benefit.
#[tracing::instrument]
fn core_process_loop(walkable_space: WalkDir, re: &Regex, args: &Args) -> Result<()> {
    let rep = &args.replacement;
    let is_test_run = args.test_run;
    let mut num_matches: u64 = 0;

    for entry in walkable_space {
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
        num_matches += 1;
        // Guard: no replacement
        let Some(rep) = rep else {
            println!("Match found: {}", &entry.black().bold().on_green());
            continue;
        };
        let new_filename = re.replace(entry, rep);
        // Guard: --test-run
        if is_test_run {
            println!("--test-run mapping: {} ~~> {}",
                     &entry.black().bold().on_green(),
                     &new_filename.red().bold().on_blue());
            continue;
        }
        println!("Renaming: {} ~~> {}", &entry.black().bold().on_green(), &new_filename.red().bold().on_blue());
        std::fs::rename(entry, new_filename.as_ref())?;
    }
    println!("Total matches: {}", num_matches.cyan());
    Ok(())
}

/// Guard: Flagging unintended syntax
///
/// Checks replacement string for capture references making a common syntax error:
/// A bare reference number followed by chars that would be combined with it and read as a name
///
/// e.g. `$1abc` will be parsed as ($1abc) NOT ($1)(abc) -- `${1}abc` is proper syntax
#[tracing::instrument]
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
#[tracing::instrument]
fn walkdir_build_with_depths(does_recurse: bool) -> WalkDir {
    if does_recurse {
        tracing::debug!("Recursable WalkDir");
        return WalkDir::new(".").min_depth(1);
    }

    tracing::debug!("non-recursing (shallow) WalkDir");
    WalkDir::new(".").min_depth(1).max_depth(1)
}

#[cfg(test)]
pub mod tests {
    use std::path::Path;

    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use test_log::test;

    use super::*;

    /// Generate a temporary directory
    fn utility_test_dir_gen() -> assert_fs::TempDir {
        let tempdir = assert_fs::TempDir::new().unwrap();
        let child_path = tempdir.child("foo.txt");
        child_path.touch().unwrap();
        tempdir
    }

    #[test]
    fn xp_test() {
        let dir = utility_test_dir_gen();

        // logging::tracing_subscribe_boilerplate("error");
        tracing::debug!("AAAAAaaAAAAAA!");
        println!("bl\na\nh\nb\nlahblah");

        println!("temp: {:?}", dir);
        let bap_file_path = dir.child("bap.txt");
        bap_file_path.touch().unwrap();
        println!("temp: {:?}", dir);
        let _ = dir.child("bar.txt").assert(predicate::path::missing());
        let _ = dir.child("bap.txt").assert(predicate::path::eq_file(dir.path().join(Path::new("bap.txt"))));

        // Both of the following error assert(a()) *and* asert(a().not())
        // as seen above the 'predicates' created are also deeply nested, with something like 4 `)` at the end
        // I'm going to commit and this and call it -- this is a bad path -- these crates create more difficulty than they solve
        // it *would* be easy to work around them and finish, but at the cost of doing work to solve an obfuscating framework
        // rather than working ona  areal problem
        //
        // let _ = dir.child("bap.txt").assert(predicate::path::eq_file(Path::new("bap.txt")));
        // let _ = dir.child("bap.txt").assert(predicate::path::eq_file(Path::new("bap.txt")).not());

        dir.close().unwrap();
        // assert_eq!(1, 2);
    }

    // Test the app() function
    // Test the core_process_loop() function

    /// Test the check_for_common_syntax_error() function
    #[test]
    fn test_check_for_common_syntax_error() {
        let test_cases = vec![("$1abc", true),
                              ("${1}abc", false),
                              //
                              ("$1a", true),
                              ("${1}a", false),
                              //
                              ("$1", false),
                              ("${1}", false),
                              //
                              ("$1 ", false),
                              ("${1} ", false),
                              //
                              ("${1} abc ", false),
                              ("$1 abc ", false),
                              //
                              ("$1abc$2", true),
                              ("${1}abc$2", false),
                              ("$1abc$2def", true),
                              //
                              ("${1}abc$2def", true),
                              ("$1abc${2}def", true),
                              ("${1}abc${2}def", false),
                              //
                              ("${1} $2", false),
                              ("$1$2 ", false)];
        for (input, expect_error) in test_cases {
            let result = check_for_common_syntax_error(input);
            match (result.is_err(), expect_error) {
                (true, true) => continue,
                (false, false) => continue,
                (true, false) => panic!("Expected no error for input: {}", input),
                (false, true) => panic!("Expected an error for input: {}", input),
            }
        }
    }

    // /// Test the walkdir_build_with_depths() function
    // fn test_walkdir_build_with_depths() {
    //     let walkdir = walkdir_build_with_depths(true);
    //     // assert_eq!(,);
    //     // assert_eq!(,);

    //     let walkdir = walkdir_build_with_depths(false);
    //     // assert_eq!(,);
    //     // assert_eq!(,);
    // }
}

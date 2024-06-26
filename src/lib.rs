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
    use std::{fs::{self, File},
              path::Path};

    use tempfile::TempDir;
    use test_log::test;

    use super::*;

    pub type Result<T> = core::result::Result<T, Error>;
    pub type Error = Box<dyn std::error::Error>;

    /// Generate a populated temporary directory.
    fn utility_test_dir_gen() -> Result<TempDir> {
        let dir_root = TempDir::new()?;
        // Root level files
        File::create(dir_root.path().join("file_01.txt"))?;
        File::create(dir_root.path().join("file_02.txt"))?;

        // First nested directory with files
        let dir_1 = dir_root.path().join("dir_1");
        fs::create_dir(&dir_1)?;
        File::create(dir_1.join("file_1a.txt"))?;

        // Second nested directory within the first directory
        let dir_11 = dir_1.join("dir11");
        fs::create_dir(&dir_11)?;
        File::create(dir_11.join("file_11b.txt"))?;

        Ok(dir_root)
    }

    /// Just a syntax check and familiarization test for working with tempdir and fs asserts.
    #[test]
    fn xp_test_fs() -> Result<()> {
        let dir_root = utility_test_dir_gen()?;

        // logging::tracing_subscribe_boilerplate("error");
        tracing::debug!("AAAAAaaAAAAAA!");
        println!("bl\na\nh\nb\nlahblah");

        println!("temp: {:?}", dir_root);
        let f_3 = File::create(dir_root.path().join("file_03.txt"))?;
        println!("temp: {:?}", dir_root);
        println!("file3: {:?}", f_3);

        assert!(dir_root.path().join("file_03.txt").exists());
        // #[cfg(target_os = "macos")]
        // {
        //     // NOTE: MacOS filesystem by *default* is case-*in*sensitive
        //     //       This is *not* an invariant on MacOS (despite my cfg logic)
        //     //       Nor is it the default in Linux, commonly
        //     assert!(dir_root.path().join("FiLe_03.txt").exists());
        //     assert!(dir_root.path().join("File_03.txt").exists());
        // }
        assert!(!dir_root.path().join("blahblahblah").exists());

        dir_root.close()?;
        Ok(())
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

    // #[test]
    // fn test_recursion() -> Result<()> {
    //     let temp_dir = setup_nested_files().unwrap();
    //     env::set_current_dir(&temp_dir.path())?;

    //     let args = Args { regex:       r"file.(\d+)\.txt".to_string(),
    //                       replacement: Some("changed-${1}.txt".to_string()),
    //                       recurse:     true,
    //                       test_run:    false, };

    //     app(&args);
    //     println!("temp: {:?}", temp_dir);

    //     assert!(temp_dir.path().join("changed-01.txt").exists());
    //     assert!(temp_dir.path().join("changed-02.txt").exists());
    //     assert!(temp_dir.path().join("dir1/changed-1a.txt").exists());
    //     assert!(temp_dir.path().join("dir1/dir11/changed-11b.txt").exists());
    //     Ok(())
    // }

    // #[test]
    // fn test_non_recursion() -> Result<()> {
    //     let temp_dir = setup_nested_files().unwrap();

    //     let args = Args { regex:       r"file.(\d+)\.txt".to_string(),
    //                       replacement: Some("changed-${1}.txt".to_string()),
    //                       recurse:     false,
    //                       test_run:    false, };

    //     let boop = app(&args);
    //     dbg!(&boop);

    //     assert!(temp_dir.path().join("changed-01.txt").exists());
    //     assert!(temp_dir.path().join("changed-02.txt").exists());
    //     // These files should not be renamed because recursion is disabled
    //     assert!(!temp_dir.path().join("dir1/changed-1a.txt").exists());
    //     assert!(!temp_dir.path().join("dir1/dir11/changed-11b.txt").exists());
    //     Ok(())
    // }
}

//! Bits 'n Bobs box.
//! Unused code snippets that may be useful later.

/// Checks for a literal `.` prefix on a entry
///
/// # Unused Because:o
/// Already have general regex search.  May implement if need an ergonomic 'quick path'.
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

/// Just a syntax check and familiarization test for working with tempdir and fs asserts.
#[test]
fn xp_test_fs() -> Result<()> {
    tracing::debug!("creatind tempdir");
    let dir_root = utility_test_dir_gen()?;

    tracing::debug!("debug level statement");
    println!("testing\na\nb\nc\nd\ntesting\n");

    println!("temp: {:?}", dir_root);
    let nf_0d = File::create(dir_root.path().join("new_file_0d.txt"))?;
    println!("temp: {:?}", dir_root);
    println!("new_file_0d: {:?}", nf_0d);

    assert!(!dir_root.path().join("blahblahblah").exists());
    assert!(dir_root.path().join("new_file_0d.txt").exists());
    #[cfg(target_os = "macos")]
    {
        // NOTE: MacOS filesystem by *default* is case-*in*sensitive
        //       This is *not* an invariant on MacOS (despite my cfg logic)
        //       Nor is it the default in Linux, commonly
        assert!(dir_root.path().join("New_file_0d.txt").exists());
        assert!(dir_root.path().join("nEw_FiLe_0D.tXt").exists());
    }

    dir_root.close()?;
    Ok(())
}

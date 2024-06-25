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

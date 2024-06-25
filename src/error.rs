//! Error handling for rename_files

// Early dev phase error type that can flow into standard error handling with type coercions.
pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;

//! Library Code.

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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

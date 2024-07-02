//! CLI interface to allow regex based file renaming
//!
//! # Example:
//! ```bash
//! clear; el; carr -- '(C|c)argo.*(\..*)' --rep '$1ogra$2' --test-run
//! clear; el; carr -- '(C|c)argo.*(\..*)' --rep '${1}ogra$2' --test-run
//! ```

use clap::Parser;
use rename_files::{app, error::Result, logging, Args};

fn main() -> Result<()> {
        logging::tracing_subscribe_boilerplate("warn");
        let args = Args::parse();
        app(&args)
}

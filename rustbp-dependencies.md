# General Dependencies Refs

# Workspace Layout
```toml
[workspace]
members = ["claps", "datapop", "derives"]
resolver = "2"

# [package]
# name = "exemplars"
# version = "0.1.0"
# edition = "2021"


## Bundles
- futures
- tokio --features=features,macros,fs,time
        ,sync
        ,full
        ,test-util
- tokio_stream
- tracing
- tracing-subscriber--features=env-filter,serde,json,chrono,tracing-serde 



[workspace.dependencies]
```
# Error Handling
- derive_more::error

# Logging
- tracing
 - tracing-subscriber--features=env-filter,serde,json,chrono,tracing-serde 
 - console-subscriber
   - println!("{}", entry.path().display());
- secrecy

async-backtrace
tracing-appender
tracing-error
tracing-flame
tracing-forest
tracing-timing

# Ergonomics
## Data Structure Behavior
- derive_more
- serde --features=derive
serde_json
derive_builder
derive-new
educe
strum
variantly
## Struct Populators
validator --features=derive,phone

## General Methods
- itertools
tap

# File System
- walkdir
globset
assert_fs
csv
dirs
dotenvy
tempfile
toml
walkdir

# Time
chrono --features=serde,arbitrary
hifitime --features=reqwest,serde

# http
## tokio::semaphore
backon
governor
- reqwest --features=serde_json
  - reqwest-retry

## async
dashmap

# DataFrames
-  polars --features=lazy
                ,ndarray,serde,json,docs-selection
## Surreal
# Visualization
plotters

# Parallelism (logical)
** rayon
** tokio --features=features,macros,fs,time
        ,sync
        ,full
        ,test-util
   futures
   tokio_stream

# Parsing
logos
regex
winnow --features=debug

# CLI
clap --features=derive,wrap_help
                ,env,unicode,string,debug
dialoguer --features=fuzzy-matcher,fuzzy-select,history,completion
indicatif --features=rayon,tokio

# WASM
- egui
  - eframe
tracing-wasm

# Math
faer --features=rayon,sere,perf-warn
nalgebra
ndarray --features=rayon,serde,docs,test
petgraph --features=serde,generate,quickcheck,serde_derive
rand --features=serde
raphtory --features=vectors,io,search
rustworkx-core
uom --features=
        serde,
        u8,
        u16,
        u32,
        u64,
        i64,
        complex64,
        rational64,


# Other
** enum_dispatch
** fakeit
** once_cell
** indoc


# [dev-dependencies]
# Testing
- quickcheck
  - quickcheck_macros
arbitrary --features=derive
pretty_assertions 
insta --features=
        csv,
        json,
        regex,
        serde,
        toml,
        walkdir,
        yaml,
wiremock

# Profilers
##dhat
##divan
##flamegraph

# for use with "console-subscriber"
[build]
rustflags = ["--cfg", "tokio_unstable"]

# dev_log - `cartfix` branch

- investigating why `cargo nextest` and `cargo test -- test-threads 1` successfully run test, but `cargo test` does not.
    - Separate tests are sharing temp directories and failing (for `cargo test`)
    - `TempDir::new()` should (confirmed: does) generate unique root names for each test
    - `cargo nextest` nominally runs tests in parallel, but does not seem to have the same issues 
    
- Likely: 
    ```rust
    std::env::set_current_dir(&temp_dir.path())?;
    ```
    is the culprit.  With env_var being overritten and causing tests to use the wrong path.
        - this does NOT explain why `cargo nextest` works, but would explain why `cargo test` does not. (shared env variables is even mentioned as a pitfall re: `cargo test`)
    - Option_1: find way to locally specify working directory 
    - Option_2: guard the env variable.  (I don't love it, because the goal was, and ideally continues to be, independent tests.)

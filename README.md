# File Rename Utility

I just needed to batch rename some files and figured I'd write a script with rust to operate using general regexes rather than use one of the (many) existing solutions.

This is just a 'does what I need' command, with it's purpose being to have written it more than to use it.  But it meets my needs quite well.

Regex syntax is, of course, [Rust Regex Syntax](https://docs.rs/regex/latest/regex/#syntax).
Replacements capture groups are (as in syntax guide) referenced with `$1` or `${1}` style.  The only exception is that only digits are expected (not named subgroups) and that `$`-following-digits followed by neither *non*-digits nor *non*-spaces nor *non*-`$` are not allowed -- bringing up a warning to encase the digit in `{}`.  (That read hard, but basically it foces you to wrap `${}`-like unless its human and machine unambiguous, and will warn if you didn't. )
This seems less likely to cause confusion at the cost of named capture group referencing, which seems unlikely to be useful here.


## Perf:

Most ad hoc of tests, but ... performs remarkably well against `fd`.  Curious what's going on there.

Just looking at search times: ~25ms (`rename_files`) vs ~20ms (`fd`).
Notably, this is without forcing parity in search space.  Our app (`rename_files`) is searching through the `.git/` dir, for example.  And returns about 55 more hits than `fd` when looking at "ho" (which has many hits in the .git dir).  (Can see this casually comparing results and counting by piping both with `| wc -l`)

Query: hyperfine reports ~100ms system time for `fd`, but only about ~16ms for `rename_files`.   Howeer, that system time is well less than the total time for `fd` (but not for `rename_files`).  -- I assume this is `fd` using multiple threads and `hyperfine` summing system time across them, whereas `rename_files` is single-threaded.

Plausible: file reads are a bottle neck to processing time for the simple regex, while simultaneously the M2max chip run here is able to efficiently gather.
Notably, running on another, M1, machine I saw time differences more like `50ms` vs `25ms` (`rename_files` vs `fd`).  Interesting to look at for its own sake.  And, of course, a more serious comparison would require much more varied loads.  *Still*, its exciting that so much performance can come from an implementation with no post-write optimizations and no major architecting aside from avoiding some ugly looking allocations that certain iterator methods would request.  (Which I'm curious to implement and compare with various compiler settings to see how impactful that would be.)

```bash
/coding_dirs/rust/rename_files on  master [!⇡] is 📦 v0.1.6 via 🦀 v1.81.0-nightly
❯ target/release/rename_files 'ho' --recurse | wc -l
     361

~/coding_dirs/rust/rename_files on  master [!⇡] is 📦 v0.1.6 via 🦀 v1.81.0-nightly
❯ fd --unrestricted 'ho' | wc -l
     307
```

```bash
~/coding_dirs/rust/rename_files on  master [!⇡] is 📦 v0.1.6 via 🦀 v1.81.0-nightly
❮ j bench-hyperf
Release, search-only:
hyperfine --warmup 3 "target/release/rename_files 'ho' --recurse"
Benchmark 1: target/release/rename_files 'ho' --recurse
  Time (mean ± σ):      26.1 ms ±   0.4 ms    [User: 6.7 ms, System: 17.8 ms]
  Range (min … max):    25.2 ms …  27.5 ms    105 runs

Release, search & replace, no file write:
hyperfine --warmup 3 "target/release/rename_files 'ho' --rep 'ohhoho' --recurse --test-run"
Benchmark 1: target/release/rename_files 'ho' --rep 'ohhoho' --recurse --test-run
  Time (mean ± σ):      26.6 ms ±   0.8 ms    [User: 6.9 ms, System: 18.0 ms]
  Range (min … max):    25.5 ms …  33.5 ms    106 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Comparison: fd --unrestricted, search-only:
hyperfine --warmup 3 "fd --unrestricted 'ho'"
Benchmark 1: fd --unrestricted 'ho'
  Time (mean ± σ):      21.4 ms ±   1.3 ms    [User: 15.5 ms, System: 111.7 ms]
  Range (min … max):    18.4 ms …  24.7 ms    122 runs
```

```bash
~/coding_dirs/rust/rename_files on  master [!⇡] is 📦 v0.1.6 via 🦀 v1.81.0-nightly
❯ j bench-hyperf '^(C|c)ar'
Release, search-only:
hyperfine --warmup 3 "target/release/rename_files '^(C|c)ar' --recurse"
Benchmark 1: target/release/rename_files '^(C|c)ar' --recurse
  Time (mean ± σ):      25.9 ms ±   0.4 ms    [User: 6.6 ms, System: 17.6 ms]
  Range (min … max):    25.2 ms …  27.2 ms    106 runs

Release, search & replace, no file write:
hyperfine --warmup 3 "target/release/rename_files '^(C|c)ar' --rep 'ohhoho' --recurse --test-run"
Benchmark 1: target/release/rename_files '^(C|c)ar' --rep 'ohhoho' --recurse --test-run
  Time (mean ± σ):      26.4 ms ±   0.9 ms    [User: 6.8 ms, System: 17.9 ms]
  Range (min … max):    25.1 ms …  31.9 ms    104 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Comparison: fd --unrestricted, search-only:
hyperfine --warmup 3 "fd --unrestricted '^(C|c)ar'"
Benchmark 1: fd --unrestricted '^(C|c)ar'
  Time (mean ± σ):      21.1 ms ±   1.2 ms    [User: 14.3 ms, System: 109.9 ms]
  Range (min … max):    18.9 ms …  24.3 ms    122 runs
```

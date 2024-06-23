# File Rename Utility

I just needed to batch rename some files and figured I'd write a script with rust to operate using general regexes rather than use one of the (many) existing solutions.

This is just a 'does what I need' command, with it's purpose being to have written it more than to use it.  But it meets my needs quite well.

Regex syntax is, of course, [Rust Regex Syntax](https://docs.rs/regex/latest/regex/#syntax).
Replacements capture groups are (as in syntax guide) referenced with `$1` or `${1}` style.  The only exception is that only digits are expected (not named subgroups) and digits that `$` following digits that are then followed by non-digits and non-spaces are not allowed -- bringing up a warning to encase the digit in `{}`.
(This seems less likely to cause confusion at the cost of named capture group referencing, which seems unlikely to be useful here.)

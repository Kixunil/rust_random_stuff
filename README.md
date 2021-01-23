# Various Rust utilities

This crate is a pile of random helpers, macros, extension traits... that I didn't care to write a separate crate for.
Probably nothing to do with `rand` crate.

While I will follow semver, I'm not going to try hard avoiding breaking changes.
This crate is forever unstable.

Using this crate in production is most likely a bad idea.
But feel free to copy anything you find here.
Or even better, polish it and create a separate crate.
Please notify me if you do so.

I may occasionally take things out of here into separate crates.
If I do so or I learn of someone else who did it, I will try to find some time to deprecate the equivalent in this crate and remove it later.

## What it contains today

* Helpers for displaying `Error` types.
* Helpers for displaying and logging errors in `Result`
* Checked operations on integer types returning `Result` (as opposed to `Option` - nicer error messages)

## Unsoundness policy

Unsound code will be fixed in patch versions even if it changes the API.
It's better to break your compilation than to leave your code vulnerable.
(You aren't using it in production anyway, RIGHT?!)

## Non-goals

* Joke code (but feel free to be punny if it's not misleading)
* Shit code
* Unsound code
* Code with huge, obvious, easily-fixable inefficiency
* Code causing frustration to programmers or users of their creations

## MSRV

What's available in Debian stable (currently 1.41).
Features requiring newer version may be introduced in the future.

## Contributing

Feel free to file PRs for fixes, improvements and new shit!
While this is not meant as high-quality crate, I have some minimal requirements:

* You agree to publish it under the same license
* All non-obvious functions must be documented (e.g. if your type is supposed to behave like a slice you don't need to document its `len()` method)
* Function names **MUST NEVER LIE**
* Error types must produce sensible error messages and **MUST NEVER LIE**
* You agree that I may sometimes ask you to change something based etirely on my subjective guess.
  Of course I will try to be reasonable but I may disagree with you.
* MSRV is obeyed (or use feature flag if not); feature flags will probably be named after required Rust version

## License

WTFPL

But if you're a government agent, employee, or contractor, then I don't like you.
(Of course, I mean mandatory government, I like voluntary fake governments.)

# Change Log
All notable changes to this project will be documented in this file. This change log follows the conventions of [keepachangelog.com](http://keepachangelog.com/).

## [Unreleased]
### Fixed
- fix Hitting Control-C in the REPL can cause infinite loop of exception handling #9

## [0.2.0] - 2026-02-05
- First public release

### Removed
- `make-widget-sync` - we're all async, all the time.

### Fixed
- Fixed widget maker to keep working when daylight savings switches over.

[Unreleased]: https://github.com/tesujimath/limabean/compare/0.2.0...HEAD
[0.2.0]: https://github.com/tesujimath/limabean/compare/0.1.0...0.2.0

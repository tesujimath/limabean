# Change Log

All notable changes to this project will be documented in this file. This change log follows the conventions of [keepachangelog.com](http://keepachangelog.com/).

## [Unreleased]

[commit log]: https://github.com/tesujimath/limabean/compare/0.3.2...HEAD

## [0.3.2] - 2026-03-06

### Added

- print-method for java.time.LocalDate so that EDN is emitted in the same form as read

[commit log]: https://github.com/tesujimath/limabean/compare/0.3.1...0.3.2

## [0.3.1] - 2026-03-05

### Fixed

- bump limabean-booking crate version number, which broke Rust release 0.3.0

[commit log]: https://github.com/tesujimath/limabean/compare/0.3.0...0.3.1

## [0.3.0] - 2026-03-05

### Added

- external plugins are loaded and run automatically #50
- plugin "beancount.plugins.implicit_prices" is now supported #38
- duplicate includes are allowed if the context is unchanged #49
- include totals in costs and prices, to preserve original precision #48

### Changed

- simplify how tolerance is handled in booking crate

### Fixed

- inference of cost-per-unit from posting weight is now supported #42
- inference of price-per-unit from total must be positive #47

[commit log]: https://github.com/tesujimath/limabean/compare/0.2.7...0.3.0

## [0.2.7] - 2026-02-23

### Added

-- support for globs in include pragmas #35

[commit log]: https://github.com/tesujimath/limabean/compare/0.2.6...0.2.7

## [0.2.6] - 2026-02-20

### Added

- plugin "beancount.plugins.auto_accounts" is now supported
- warn about unknown plugins

### Fixed

- fix intolerance of zero #32
- fix writing of POM file broken by existing pom.xml

[commit log]: https://github.com/tesujimath/limabean/compare/0.2.5...0.2.6

## [0.2.5] - 2026-02-17

### Added

- Build uberjar again for standalone use #28

[commit log]: https://github.com/tesujimath/limabean/compare/0.2.4...0.2.5

## [0.2.4] - 2026-02-13

### Fixed

- fix Parsing Beancount files with CR-LF is failing on Windows #24

[commit log]: https://github.com/tesujimath/limabean/compare/0.2.3...0.2.4

## [0.2.3] - 2026-02-10

### Added

- implement show for set #10
- implement show for seq

### Fixed

- fix Unable to detect a system Terminal on Windows #5
- fix Weird terminal behaviour and Control-C handling on Windows #11

[commit log]: https://github.com/tesujimath/limabean/compare/0.2.1...0.2.3

## [0.2.2] - 2026-02-10

Broken release, do not use

## [0.2.1] - 2026-02-07

### Fixed

- fix Hitting Control-C in the REPL can cause infinite loop of exception handling #9

[commit log]: https://github.com/tesujimath/limabean/compare/0.2.0...0.2.1

## [0.2.0] - 2026-02-05

First public release

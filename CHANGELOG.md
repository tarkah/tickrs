<!-- Disable MD024 because `Keep a Changelog` use duplicate
header titles -->
<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The sections should follow the order `Packaging`, `Added`, `Changed`, `Fixed`
and `Removed`.

## [Unreleased]

### Added

- Custom themes can now be applied. See the [themes wiki] entry for more
  information ([#69])

## [0.11.0] - 2021-02-12

### Added

- Summary pane can be scrolled with Up / Down arrows if more tickers are present
  than are able to be shown in the terminal ([#63])
- A config file can now be used to change program behavior. A default file will
  be created / can be updated at the following locations ([#66])
  - Linux: `$HOME/.config/tickrs/config.yml`
  - macOS: `$HOME/Library/Application Support/tickrs/config.yml`
  - Windows: `%APPDATA%\tickrs\config.yml`
- Current tab can be reordered by using `Ctrl + Left / Right` ([#67])

## [0.10.2] - 2021-02-10

### Fixed

- Fixed bug that would deadlock the program between 12am - 4am ET on the intraday
  1D timeframe ([#59])

## [0.10.1] - 2021-02-08

### Fixed

- Options pane now re-renders correctly when resizing terminal window ([#57])
- Prevent application from crashing when terminal was too small with options pane
  open ([#57])

## [0.10.0] - 2021-02-08

### Fixed

- Huge improvements to optimization of program. CPU usage is way down ([#54])
- Fix 1W - 6M time frame graphing for Crypto tickers where not all datapoints
  were plotted correctly across the x-axis ([#55])

## [0.9.1] - 2021-02-06

### Changed

- Help page can be exited with `q` key ([#51])
- Added a note to help page about options not being enabled for crypto ([#50])

### Fixed

- Stocks that IPOd more recently than selected timeframe no longer stretch the
  entire x-axis width and now start plotting at the correct spot ([#48])
- Fix bug where too many file descriptors are opened due to recreating http
  client ([#53])

## [0.9.0] - 2021-02-04

### Added

- Added support for graphing volumes. You can press `v` to toggle volumes

### Fixed

- Fixed issue on 1D graph with pre / post enabled where missing datapoints caused
  line to not reach end of x-axis by end of day. Now line always reaches end of
  x-axis


[#48]: https://github.com/tarkah/tickrs/pull/48
[#50]: https://github.com/tarkah/tickrs/pull/50
[#51]: https://github.com/tarkah/tickrs/pull/51
[#53]: https://github.com/tarkah/tickrs/pull/53
[#54]: https://github.com/tarkah/tickrs/pull/54
[#55]: https://github.com/tarkah/tickrs/pull/55
[#57]: https://github.com/tarkah/tickrs/pull/57
[#59]: https://github.com/tarkah/tickrs/pull/59
[#63]: https://github.com/tarkah/tickrs/pull/63
[#66]: https://github.com/tarkah/tickrs/pull/66
[#67]: https://github.com/tarkah/tickrs/pull/67
[#69]: https://github.com/tarkah/tickrs/pull/69
[themes wiki]: https://github.com/tarkah/tickrs/wiki/Themes
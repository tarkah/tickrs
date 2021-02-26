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

### Fixed

- Fixed keybind to correctly capture <kbd>SHIFT</kbd>+<kbd>TAB</kbd> in the
  chart configuration pane

### Changed

- UI changes so that wording is consistent throughout ([#112])
  - Stock symbols show as uppercase in tabs section
  - Letters for stock information are now capitalized
  - Words in Options pane are now capitalized
  - Toggle box shows the current chart type rather than the next chart type

## [0.14.0] - 2021-02-26

### Added

- Kagi charts have been added! ([#93])
  - You can specify custom reversal type (pct or amount), reversal value, and
    price type (close or high_low) within the GUI by pressing 'e'
  - New config options have been added to configure the behavior of Kagi charts,
    see the updated [wiki entry](https://github.com/tarkah/tickrs/wiki/Config-file)
  - As Kagi charts x-axis is decoupled from time, the chart width may be wider than
    the terminal. You can now press <kbd>SHIFT</kbd> + <kbd><</kbd> / <kbd>></kbd>
    or <kbd>SHIFT</kbd> + <kbd>LEFT</kbd> / <kbd>RIGHT</kbd> to scroll the chart.
    An indicator in the bottom right corner will notify you if you can scroll further
    left / right
  - `--candle` has been deprecated in favor of `--chart-type`

### Packaging

- Linux: Binary size has been reduced due to some optimizations, from 10.6MB to
  8MB ([#86])

## [0.13.1] - 2021-02-22

### Fixed

- Fixed theme background not getting applied to all widgets ([#84])
- Fixed last x label for candlestick charts from showing unix time 0 for 1W - 5Y
  timeframes ([#85])

## [0.13.0] - 2021-02-19

### Added

- Candestick chart support has been added. You can press 'c' to toggle between
  line and candlestick charts ([#75])
  - You can also pass the `--candle` flag on startup, or specify `candle: true`
    in the config file to launch with candlestick charting enabled

### Changed

- All theme colors are now optional and can be selectively included / omitted from
  the theme config ([#76])

### Fixed

- Fixed panic when width of terminal was too small on main stock screen ([4cc00d0](https://github.com/tarkah/tickrs/commit/4cc00d052c4bfff993587f1342086498ee8b2766))
- Fix bug where cursor icon still shows in some terminals such as WSL2 on Windows with Alacritty ([#79])

## [0.12.0] - 2021-02-17

### Added

- Custom themes can now be applied. See the [themes wiki](https://github.com/tarkah/tickrs/wiki/Themes) entry for more
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
[#75]: https://github.com/tarkah/tickrs/pull/75
[#76]: https://github.com/tarkah/tickrs/pull/76
[#79]: https://github.com/tarkah/tickrs/pull/79
[#84]: https://github.com/tarkah/tickrs/pull/84
[#85]: https://github.com/tarkah/tickrs/pull/85
[#86]: https://github.com/tarkah/tickrs/pull/86
[#93]: https://github.com/tarkah/tickrs/pull/93
[#112]: https://github.com/tarkah/tickrs/pull/112

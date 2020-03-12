# tick-rs
[![Build Status](https://dev.azure.com/tarkah/tickrs/_apis/build/status/tarkah.tickrs?branchName=master)](https://dev.azure.com/tarkah/tickrs/_build/latest?definitionId=17&branchName=master)

Realtime ticker data in your terminal 📈 Built with Rust.

  - [Installation](#installation)
  - [Usage](#usage)
    - [Windows](#windows)
  - [Acknowledgments](#acknowledgments)

*WIP*

![screenshot](assets/screenshot.png)


## Installation

### Binary

Download the latest [release](https://github.com/tarkah/tickrs/releases/latest) for your platform

## Usage

```
tickrs 0.1.0
Realtime ticker data in your terminal 📈

USAGE:
    tickrs [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
        --hide-help    Hide help icon in top right
    -V, --version      Prints version information

OPTIONS:
    -s, --symbols <symbols>...    Comma separated list of ticker symbols to start app with
```

### Windows

Use [Windows Terminal](https://www.microsoft.com/en-us/p/windows-terminal-preview/9n0dx20hk701) to properly display this app.

## Acknowledgments
- [antoinevulcain](https://github.com/antoinevulcain) / [Financial-Modeling-Prep-API](https://github.com/antoinevulcain/Financial-Modeling-Prep-API) - for the awesome API powering this app, [https://financialmodelingprep.com](https://financialmodelingprep.com)
- [fdehau](https://github.com/fdehau) / [tui-rs](https://github.com/fdehau/tui-rs) - great TUI library for Rust
- [cjbassi](https://github.com/cjbassi) / [ytop](https://github.com/cjbassi/ytop) - thanks for the inspiration!
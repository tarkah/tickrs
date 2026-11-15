use std::collections::HashMap;
use std::{fs, process};

use anyhow::{bail, format_err, Error};
use serde::Deserialize;
use structopt::StructOpt;

use crate::common::{ChartType, TimeFrame};
use crate::portfolio::Portfolio;
use crate::theme::Theme;
use crate::widget::KagiOptions;

pub fn resolve_opts() -> Opts {
    let mut opts = get_cli_opts();

    if let Err(error) =
        config_dir().and_then(|dir| apply_saved_symbols(&mut opts, &dir.join("symbols.yml")))
    {
        eprintln!("Could not load saved symbols: {error}");
    }

    if let Ok(config_opts) = get_config_opts() {
        // Options
        opts.chart_type = opts.chart_type.or(config_opts.chart_type);
        opts.symbols = opts.symbols.or(config_opts.symbols);
        opts.time_frame = opts.time_frame.or(config_opts.time_frame);
        opts.update_interval = opts.update_interval.or(config_opts.update_interval);

        // Flags
        opts.enable_pre_post = opts.enable_pre_post || config_opts.enable_pre_post;
        opts.hide_help = opts.hide_help || config_opts.hide_help;
        opts.hide_prev_close = opts.hide_prev_close || config_opts.hide_prev_close;
        opts.hide_toggle = opts.hide_toggle || config_opts.hide_toggle;
        opts.show_volumes = opts.show_volumes || config_opts.show_volumes;
        opts.show_x_labels = opts.show_x_labels || config_opts.show_x_labels;
        opts.summary = opts.summary || config_opts.summary;
        opts.trunc_pre = opts.trunc_pre || config_opts.trunc_pre;

        // Theme
        opts.theme = config_opts.theme;

        // Kagi Options
        opts.kagi_options = config_opts.kagi_options;

        // Portfolio
        opts.portfolio = config_opts.portfolio;
    }

    opts
}

fn get_cli_opts() -> Opts {
    Opts::from_args()
}

fn config_dir() -> Result<std::path::PathBuf, Error> {
    Ok(dirs_next::config_dir()
        .ok_or_else(|| format_err!("Could not get config directory"))?
        .join("tickrs"))
}

fn apply_saved_symbols(opts: &mut Opts, path: &std::path::Path) -> Result<(), Error> {
    if opts.symbols.is_none() {
        opts.symbols = load_saved_symbols(path)?;
    }
    Ok(())
}

fn load_saved_symbols(path: &std::path::Path) -> Result<Option<Vec<String>>, Error> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(Some(serde_yaml::from_str(&contents)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn save_symbols(symbols: &[String]) -> Result<(), Error> {
    save_symbols_to(&config_dir()?.join("symbols.yml"), symbols)
}

fn save_symbols_to(path: &std::path::Path, symbols: &[String]) -> Result<(), Error> {
    let contents = serde_yaml::to_string(symbols)?;
    let parent = path
        .parent()
        .ok_or_else(|| format_err!("Missing config directory"))?;
    fs::create_dir_all(parent)?;
    // Write beside the destination and rename so an interrupted write cannot
    // truncate the previous saved list.
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> Result<(), Error> {
        use std::io::Write;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn get_config_opts() -> Result<Opts, Error> {
    let config_dir = config_dir()?;

    if !config_dir.exists() {
        let _ = fs::create_dir_all(&config_dir);
    }

    let config_path = config_dir.join("config.yml");

    if !config_path.exists() {
        let _ = fs::write(&config_path, DEFAULT_CONFIG);
    }

    let config = fs::read_to_string(&config_path)?;

    let opts = match serde_yaml::from_str::<Option<Opts>>(&config) {
        Ok(Some(opts)) => opts,
        Ok(None) => bail!("Empty config file"),
        Err(e) => {
            println!(
                "Error parsing config file, make sure format is valid\n\n  {}",
                e
            );
            process::exit(1);
        }
    };

    Ok(opts)
}

#[derive(Debug, StructOpt, Clone, Deserialize, Default)]
#[structopt(
    name = "tickrs",
    about = "Realtime ticker data in your terminal 📈",
    version = env!("CARGO_PKG_VERSION")
)]
#[serde(default)]
pub struct Opts {
    // Options
    //
    #[structopt(short, long, possible_values(&["line", "candle", "kagi"]))]
    /// Chart type to start app with [default: line]
    pub chart_type: Option<ChartType>,
    #[structopt(short, long, use_delimiter = true)]
    /// Comma separated list of ticker symbols to start app with
    pub symbols: Option<Vec<String>>,
    #[structopt(short = "t", long, possible_values(&["1D", "1W", "1M", "3M", "6M", "1Y", "5Y"]))]
    /// Use specified time frame when starting program and when new stocks are added [default: 1D]
    pub time_frame: Option<TimeFrame>,
    #[structopt(short = "i", long)]
    /// Interval to update data from API (seconds) [default: 1]
    pub update_interval: Option<u64>,

    // Flags
    //
    #[structopt(short = "p", long)]
    /// Enable pre / post market hours for graphs
    pub enable_pre_post: bool,
    #[structopt(long)]
    /// Hide help icon in top right
    pub hide_help: bool,
    #[structopt(long)]
    /// Hide previous close line on 1D chart
    pub hide_prev_close: bool,
    #[structopt(long)]
    /// Hide toggle block
    pub hide_toggle: bool,
    #[structopt(long)]
    /// Show volumes graph
    pub show_volumes: bool,
    #[structopt(short = "x", long)]
    /// Show x-axis labels
    pub show_x_labels: bool,
    #[structopt(long)]
    /// Start in summary mode
    pub summary: bool,
    #[structopt(long)]
    /// Truncate pre market graphing to only 30 minutes prior to markets opening
    pub trunc_pre: bool,

    #[structopt(skip)]
    pub theme: Option<Theme>,
    #[structopt(skip)]
    pub kagi_options: HashMap<String, KagiOptions>,
    #[structopt(skip)]
    pub portfolio: Option<Portfolio>,
}

const DEFAULT_CONFIG: &str = "---
# List of ticker symbols to start app with
#symbols:
#  - SPY
#  - AMD

# Chart type to start app with
# Default is line
# Possible values: line, candle, kagi
#chart_type: candle

# Use specified time frame when starting program and when new stocks are added
# Default is 1D
# Possible values: 1D, 1W, 1M, 3M, 6M, 1Y, 5Y
#time_frame: 1D

# Interval to update data from API (seconds)
# Default is 1
#update_interval: 1

# Enable pre / post market hours for graphs
#enable_pre_post: true

# Hide help icon in top right
#hide_help: true

# Hide previous close line on 1D chart
#hide_prev_close: true

# Hide toggle block
#hide_toggle: true

# Show volumes graph
#show_volumes: true

# Show x-axis labels
#show_x_labels: true

# Start in summary mode
#summary: true

# Truncate pre market graphing to only 30 minutes prior to markets opening
#trunc_pre: true

# Ticker options for Kagi charts
#
# A map of each ticker with reversal and/or price fields (both optional). If no
# entry is defined for a symbol, a default of 'close' price and 1% for 1D and 4%
# for non-1D timeframes is used. This can be updated in the GUI by pressing 'e'
#
# reversal can be supplied as a single value, or a map on time frame to give each
# time frame a different reversal amount
#
# reversal.type can be 'amount' or 'pct'
#
# price can be 'close' or 'high_low'
#
#kagi_options:
#  SPY:
#    reversal:
#      type: amount
#      value: 5.00
#    price: close
#  AMD:
#    price: high_low
#  TSLA:
#    reversal:
#      type: pct
#      value: 0.08
#  NVDA:
#    reversal:
#      1D:
#        type: pct
#        value: 0.02
#      5Y:
#        type: pct
#        value: 0.10

# Apply a custom theme
#
# All colors are optional. If commented out / omitted, the color will get sourced
# from your terminal color scheme
#theme:
#  background: '#403E41'
#  gray: '#727072'
#  profit: '#ADD977'
#  loss: '#FA648A'
#  text_normal: '#FCFCFA'
#  text_primary: '#FFDA65'
#  text_secondary: '#79DBEA'
#  border_primary: '#FC9766'
#  border_secondary: '#FCFCFA'
#  border_axis: '#FC9766'
#  highlight_focused: '#FC9766'
#  highlight_unfocused: '#727072'

# Portfolio tracking
#
# Track your portfolio holdings to see profit/loss
# Each ticker is tracked with quantity and average purchase price
# Ticker symbols must match those in the symbols list in order to be visible
#
#portfolio:
#  AMD:
#    quantity: 1.3
#    average_price: 221
#  SPY:
#    quantity: 100
#    average_price: 450.25
";

#[cfg(test)]
mod saved_symbols_tests {
    use super::*;

    #[test]
    fn round_trip_preserves_order_empty_list_and_config() {
        let dir = std::env::temp_dir().join(format!("tickrs-symbols-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let config = dir.join("config.yml");
        let original = "# Keep my comments\nchart_type: candle\n";
        fs::write(&config, original).unwrap();
        let path = dir.join("symbols.yml");
        assert_eq!(load_saved_symbols(&path).unwrap(), None);
        let symbols = vec!["MAXX.CN".into(), "QIMC.CN".into()];
        save_symbols_to(&path, &symbols).unwrap();
        assert_eq!(load_saved_symbols(&path).unwrap(), Some(symbols.clone()));
        let mut opts = Opts::default();
        apply_saved_symbols(&mut opts, &path).unwrap();
        assert_eq!(opts.symbols, Some(symbols));
        let cli_symbols = vec!["SPY".into()];
        opts.symbols = Some(cli_symbols.clone());
        apply_saved_symbols(&mut opts, &path).unwrap();
        assert_eq!(opts.symbols, Some(cli_symbols));
        save_symbols_to(&path, &[]).unwrap();
        assert_eq!(load_saved_symbols(&path).unwrap(), Some(vec![]));
        let mut opts = Opts::default();
        apply_saved_symbols(&mut opts, &path).unwrap();
        let configured_symbols = Some(vec!["SPY".into()]);
        assert_eq!(opts.symbols.or(configured_symbols), Some(vec![]));
        assert_eq!(fs::read_to_string(config).unwrap(), original);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn invalid_saved_symbols_are_reported() {
        let path =
            std::env::temp_dir().join(format!("tickrs-invalid-symbols-{}", std::process::id()));
        fs::write(&path, "symbols: [broken").unwrap();
        assert!(load_saved_symbols(&path).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn write_failure_is_reported() {
        let path =
            std::env::temp_dir().join(format!("tickrs-blocked-symbols-{}", std::process::id()));
        fs::write(&path, "not a directory").unwrap();
        assert!(save_symbols_to(&path.join("symbols.yml"), &[]).is_err());
        fs::remove_file(path).unwrap();
    }
}

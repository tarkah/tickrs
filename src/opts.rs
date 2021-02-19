use std::{fs, process};

use anyhow::{bail, format_err, Error};
use serde::Deserialize;
use structopt::StructOpt;

use crate::common::TimeFrame;
use crate::theme::Theme;

pub fn resolve_opts() -> Opts {
    let mut opts = get_cli_opts();

    if let Ok(config_opts) = get_config_opts() {
        // Options
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
    }

    opts
}

fn get_cli_opts() -> Opts {
    Opts::from_args()
}

fn get_config_opts() -> Result<Opts, Error> {
    let config_dir = dirs_next::config_dir()
        .ok_or_else(|| format_err!("Could not get config directory"))?
        .join("tickrs");

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
}

const DEFAULT_CONFIG: &str = "---
# List of ticker symbols to start app with
#symbols:
#  - SPY
#  - AMD

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
";

use std::fs;

use anyhow::{format_err, Error};
use serde::Deserialize;
use structopt::StructOpt;

use crate::common::TimeFrame;

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

    let config_path = config_dir.join("config.yml");

    let config = fs::read_to_string(&config_path)?;

    let opts = serde_yaml::from_str(&config)?;

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
}

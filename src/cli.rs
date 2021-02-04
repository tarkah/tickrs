use structopt::StructOpt;

use crate::common::TimeFrame;

#[derive(Debug, StructOpt, Clone)]
#[structopt(
    name = "tickrs",
    about = "Realtime ticker data in your terminal 📈",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Opt {
    #[structopt(short, long, use_delimiter = true)]
    /// Comma separated list of ticker symbols to start app with
    pub symbols: Vec<String>,
    #[structopt(long)]
    /// Hide help icon in top right
    pub hide_help: bool,
    #[structopt(long)]
    /// Hide toggle block
    pub hide_toggle: bool,
    #[structopt(long)]
    /// Hide previous close line on 1D chart
    pub hide_prev_close: bool,
    #[structopt(long)]
    /// Start in summary mode
    pub summary: bool,
    #[structopt(short = "i", long, default_value = "1")]
    /// Interval to update data from API (seconds)
    pub update_interval: u64,
    #[structopt(short = "t", long, default_value = "1D", possible_values(&["1D", "1W", "1M", "3M", "6M", "1Y", "5Y"]))]
    /// Use specified time frame when starting program and when new stocks are added
    pub time_frame: TimeFrame,
    #[structopt(short = "x", long)]
    /// Show x-axis labels
    pub show_x_labels: bool,
    #[structopt(short = "p", long)]
    /// Enable pre / post market hours for graphs
    pub enable_pre_post: bool,
    #[structopt(long)]
    /// Truncate pre market graphing to only 30 minutes prior to markets opening
    pub trunc_pre: bool,
    #[structopt(long)]
    /// Show volumes graph
    pub show_volumes: bool,
}

pub fn get_opts() -> Opt {
    Opt::from_args()
}

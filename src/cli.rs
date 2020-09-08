use structopt::StructOpt;

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
    /// Start in summary mode
    pub summary: bool,
    #[structopt(short = "i", long, default_value = "1")]
    /// Interval to update data from API (seconds)
    pub update_interval: u64,
}

pub fn get_opts() -> Opt {
    Opt::from_args()
}

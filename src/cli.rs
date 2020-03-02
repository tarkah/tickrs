use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "tickrs",
    about = "Get realtime ticker data in your console",
    version = "0.1.0"
)]
pub struct Opt {
    #[structopt(short, long, use_delimiter = true)]
    /// Comma separated list of ticker symbols to start app with
    pub symbols: Vec<String>,
    #[structopt(long)]
    /// Hide help icon in top right
    pub hide_help: bool,
}

pub fn get_opts() -> Opt {
    Opt::from_args()
}

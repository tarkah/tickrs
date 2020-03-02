mod client;
pub mod model;

pub use client::Client;

#[derive(Copy, Clone)]
pub enum Interval {
    Minute1,
    Minute5,
    Minute15,
    Minute30,
    Hourly,
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Interval::Minute1 => "1min",
            Interval::Minute5 => "5min",
            Interval::Minute15 => "15min",
            Interval::Minute30 => "30min",
            Interval::Hourly => "1hour",
        };

        write!(f, "{}", s)
    }
}

mod client;
pub mod model;

pub use client::Client;

#[derive(Copy, Clone)]
pub enum Interval {
    Minute1,
    Minute2,
    Minute5,
    Minute15,
    Minute30,
    Minute60,
    Minute90,
    Hour1,
    Day1,
    Day5,
    Week1,
    Month1,
    Month3,
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Interval::*;

        let s = match self {
            Minute1 => "1m",
            Minute2 => "2m",
            Minute5 => "5m",
            Minute15 => "15m",
            Minute30 => "30m",
            Minute60 => "60m",
            Minute90 => "90m",
            Hour1 => "1h",
            Day1 => "1d",
            Day5 => "5d",
            Week1 => "1wk",
            Month1 => "1mo",
            Month3 => "3mo",
        };

        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone)]
pub enum Range {
    Day1,
    Day5,
    Month1,
    Month3,
    Month6,
    Year1,
    Year2,
    Year5,
    Year10,
    Ytd,
    Max,
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Range::*;

        let s = match self {
            Day1 => "1d",
            Day5 => "5d",
            Month1 => "1mo",
            Month3 => "3mo",
            Month6 => "6mo",
            Year1 => "1y",
            Year2 => "2y",
            Year5 => "5y",
            Year10 => "10y",
            Ytd => "ytd",
            Max => "max",
        };

        write!(f, "{}", s)
    }
}

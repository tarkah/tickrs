use chrono::{Local, NaiveDate};
use std::time::Duration;

#[derive(PartialEq, Clone, Copy, PartialOrd)]
pub enum TimeFrame {
    Day1,
    Week1,
    Month1,
    Month3,
    Month6,
    Year1,
    Year5,
}

impl TimeFrame {
    pub fn as_of_date(self) -> NaiveDate {
        let today = Local::today().naive_local();

        match self {
            TimeFrame::Day1 => today,
            TimeFrame::Week1 => today - chrono::Duration::days(6),
            TimeFrame::Month1 => today - chrono::Duration::days(30),
            TimeFrame::Month3 => today - chrono::Duration::days(30 * 3),
            TimeFrame::Month6 => today - chrono::Duration::days(30 * 6),
            TimeFrame::Year1 => today - chrono::Duration::days(365),
            TimeFrame::Year5 => today - chrono::Duration::days(365 * 5),
        }
    }

    pub fn idx(self) -> usize {
        match self {
            TimeFrame::Day1 => 0,
            TimeFrame::Week1 => 1,
            TimeFrame::Month1 => 2,
            TimeFrame::Month3 => 3,
            TimeFrame::Month6 => 4,
            TimeFrame::Year1 => 5,
            TimeFrame::Year5 => 6,
        }
    }

    pub const fn tab_names() -> [&'static str; 7] {
        ["1D", "1W", "1M", "3M", "6M", "1Y", "5Y"]
    }

    pub fn update_interval(self) -> Duration {
        match self {
            TimeFrame::Day1 => Duration::from_secs(60),
            TimeFrame::Week1 => Duration::from_secs(60 * 60),
            TimeFrame::Month1 => Duration::from_secs(60 * 60 * 24),
            TimeFrame::Month3 => Duration::from_secs(60 * 60 * 24),
            TimeFrame::Month6 => Duration::from_secs(60 * 60 * 24),
            TimeFrame::Year1 => Duration::from_secs(60 * 60 * 24),
            TimeFrame::Year5 => Duration::from_secs(60 * 60 * 24),
        }
    }

    pub fn up(self) -> TimeFrame {
        match self {
            TimeFrame::Day1 => TimeFrame::Week1,
            TimeFrame::Week1 => TimeFrame::Month1,
            TimeFrame::Month1 => TimeFrame::Month3,
            TimeFrame::Month3 => TimeFrame::Month6,
            TimeFrame::Month6 => TimeFrame::Year1,
            TimeFrame::Year1 => TimeFrame::Year5,
            TimeFrame::Year5 => TimeFrame::Day1,
        }
    }

    pub fn down(self) -> TimeFrame {
        match self {
            TimeFrame::Day1 => TimeFrame::Year5,
            TimeFrame::Week1 => TimeFrame::Day1,
            TimeFrame::Month1 => TimeFrame::Week1,
            TimeFrame::Month3 => TimeFrame::Month1,
            TimeFrame::Month6 => TimeFrame::Month3,
            TimeFrame::Year1 => TimeFrame::Month6,
            TimeFrame::Year5 => TimeFrame::Year1,
        }
    }
}

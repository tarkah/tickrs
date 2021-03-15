pub mod default_timestamps;
pub mod options;
pub mod stock;

/// Container of one or more tasks, that manages capturing all queued task responses
/// into one update response
pub trait Service {
    type Update;

    fn updates(&self) -> Vec<Self::Update>;

    fn pause(&self);

    fn resume(&self);
}

pub mod stock;

/// Container of one or more tasks
pub trait Service {
    type Update;

    fn updates(&self) -> Vec<Self::Update>;
}

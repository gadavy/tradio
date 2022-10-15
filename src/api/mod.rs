use futures::future::BoxFuture;

pub use radio_browser::RadioBrowser;

use crate::models::{Station, StationsFilter};

mod radio_browser;

pub trait Client: Sync + Send {
    fn search(&self, filter: &StationsFilter) -> BoxFuture<anyhow::Result<Vec<Station>>>;
}

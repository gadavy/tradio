pub use radio_browser::RadioBrowser;

use crate::models::{Station, StationsFilter};

mod radio_browser;

pub trait Client: Sync + Send {
    fn name(&self) -> &str;

    async fn search(&self, filter: &StationsFilter) -> anyhow::Result<Vec<Station>>;
}

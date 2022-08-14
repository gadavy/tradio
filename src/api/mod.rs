use futures::future::BoxFuture;

pub use radio_browser::RadioBrowser;

use crate::models::Station;

mod radio_browser;

pub trait Client {
    fn stations(&self) -> BoxFuture<anyhow::Result<Vec<Station>>>;
}

use futures::future::BoxFuture;

pub use sqlite::Sqlite;

use crate::models::{Station, StationsFilter};

mod sqlite;

pub trait Storage {
    /// Store new `Station` to database and returns id.
    fn create(&self, station: &Station) -> BoxFuture<anyhow::Result<i64>>;

    /// Search stations by filter.
    fn search(&self, filter: StationsFilter) -> BoxFuture<anyhow::Result<Vec<Station>>>;

    /// Update current `Station` in database.
    fn update(&self, station: &Station) -> BoxFuture<anyhow::Result<()>>;

    /// Remove `Station` from database by id.
    fn delete(&self, station_id: i64) -> BoxFuture<anyhow::Result<()>>;
}

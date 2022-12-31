pub use sqlite::Sqlite;

use crate::models::{Station, StationsFilter};

mod sqlite;

pub trait Storage: Sync + Send {
    /// Store new [Station] to database and returns id.
    async fn create(&self, station: &Station) -> anyhow::Result<i64>;

    /// Search stations by filter.
    async fn search(&self, filter: &StationsFilter) -> anyhow::Result<Vec<Station>>;

    /// Update current [Station] in database.
    async fn update(&self, station: &Station) -> anyhow::Result<()>;

    /// Remove [Station] from database by id.
    async fn delete(&self, station_id: i64) -> anyhow::Result<()>;
}

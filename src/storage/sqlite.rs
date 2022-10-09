use std::str::FromStr;
use std::time::SystemTime;

use futures::future::BoxFuture;
use futures::TryStreamExt;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::Row;

use super::{Station, StationsFilter, Storage};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: SqlitePool,
}

impl Sqlite {
    pub async fn new(url: &str) -> anyhow::Result<Sqlite> {
        let opts = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await?;

        MIGRATOR.run(&pool).await?;

        Ok(Self { pool })
    }
}

impl Storage for Sqlite {
    fn create(&self, station: &Station) -> BoxFuture<anyhow::Result<i64>> {
        let query = sqlx::query(
            r#"INSERT INTO radio_stations (
            created_at,
            updated_at,
            external_id,
            name,
            url,
            codec,
            bitrate,
            tags,
            country
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            ?5,
            ?6,
            ?7,
            ?8,
            ?9
        ) RETURNING id"#,
        )
        .bind(DateTime::<Utc>::from(SystemTime::now()))
        .bind(DateTime::<Utc>::from(SystemTime::now()))
        .bind(station.external_id.clone())
        .bind(station.name.clone())
        .bind(station.url.clone())
        .bind(station.codec.clone())
        .bind(station.bitrate)
        .bind(station.tags.to_string())
        .bind(station.country.clone());

        let pool = self.pool.clone();

        Box::pin(async move {
            let row = query.fetch_one(&pool).await?;

            Ok(row.get("id"))
        })
    }

    fn search(&self, _filter: &StationsFilter) -> BoxFuture<anyhow::Result<Vec<Station>>> {
        let query = sqlx::query(
            r#"SELECT
                id,
                created_at,
                updated_at,
                external_id,
                name,
                url,
                codec,
                bitrate,
                tags,
                country
            FROM radio_stations"#,
        );

        let pool = self.pool.clone();

        Box::pin(async move {
            let mut rows = query.fetch(&pool);

            let mut result = vec![];

            while let Some(row) = rows.try_next().await? {
                result.push(Station {
                    id: row.try_get("id")?,
                    external_id: row.try_get("external_id")?,
                    name: row.try_get("name")?,
                    url: row.try_get("url")?,
                    codec: row.try_get("codec")?,
                    bitrate: row.try_get("bitrate")?,
                    tags: row.try_get::<'_, String, _>("tags")?.into(),
                    country: row.try_get("country")?,
                });
            }

            Ok(result)
        })
    }

    fn update(&self, station: &Station) -> BoxFuture<anyhow::Result<()>> {
        let query = sqlx::query(
            r#"UPDATE radio_stations SET
                updated_at = ?1,
                external_id = ?2,
                name = ?3,
                url = ?4,
                codec = ?5,
                bitrate = ?6,
                tags = ?7,
                country = ?8
            WHERE id = ?9"#,
        )
        .bind(DateTime::<Utc>::from(SystemTime::now()))
        .bind(station.external_id.clone())
        .bind(station.name.to_string())
        .bind(station.url.to_string())
        .bind(station.codec.to_string())
        .bind(station.bitrate)
        .bind(station.tags.to_string())
        .bind(station.country.to_string())
        .bind(station.id);

        let pool = self.pool.clone();

        Box::pin(async move {
            query.execute(&pool).await?;

            Ok(())
        })
    }

    fn delete(&self, station_id: i64) -> BoxFuture<anyhow::Result<()>> {
        let query =
            sqlx::query("DELETE FROM radio_stations WHERE id = ?1").bind(station_id.to_string());
        let pool = self.pool.clone();

        Box::pin(async move {
            query.execute(&pool).await?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{Sqlite, Station, StationsFilter, Storage};

    #[tokio::test]
    async fn create() {
        let db = Sqlite::new(":memory:").await.unwrap();
        let stations = vec![new_station(1, true), new_station(2, false)];

        for station in stations {
            db.create(&station).await.unwrap();
        }
    }

    #[tokio::test]
    async fn update() {
        let db = Sqlite::new(":memory:").await.unwrap();
        let stations = vec![new_station(1, true), new_station(2, false)];

        for station in stations {
            db.create(&station).await.unwrap();
        }
    }

    #[tokio::test]
    async fn crud() {
        let db = Sqlite::new(":memory:").await.unwrap();
        let mut station = new_station(1, true);

        station.id = db.create(&station).await.unwrap();
        assert_eq!(station.id, 1);

        let stations = db.search(StationsFilter::default()).await.unwrap();
        assert_eq!(stations, vec![station.clone()]);

        let new_station = new_station(1, true);

        db.update(&new_station).await.unwrap();

        let stations = db.search(StationsFilter::default()).await.unwrap();
        assert_eq!(stations, vec![new_station.clone()]);

        db.delete(station.id).await.unwrap();

        let stations = db.search(StationsFilter::default()).await.unwrap();
        assert_eq!(stations, vec![]);
    }

    fn new_station(id: i64, with_external_id: bool) -> Station {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Station {
            id,
            external_id: with_external_id.then(|| format!("{}", id)),
            name: format!("name_{}_{}", now_secs, id),
            url: format!("url_{}_{}", now_secs, id),
            codec: format!("codec_{}_{}", now_secs, id),
            bitrate: id as u32,
            tags: "a,b,c,d,e,f".into(),
            country: format!("country_{}_{}", now_secs, id),
        }
    }
}

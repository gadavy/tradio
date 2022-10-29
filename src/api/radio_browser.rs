use std::sync::Arc;

use anyhow::Context;
use futures::future::BoxFuture;
use reqwest::{redirect::Policy, ClientBuilder, Url};
use serde::{de::DeserializeOwned, Deserialize};

use crate::models::{OrderBy, Station, StationsFilter};

use super::Client;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const RADIO_BROWSER_NAME: &str = "radio-browser";

#[derive(Debug, Clone)]
pub struct RadioBrowser {
    addr: Arc<Url>,
    client: reqwest::Client,
}

impl RadioBrowser {
    pub fn new() -> Self {
        let addr = Arc::new(
            "https://de1.api.radio-browser.info"
                .parse()
                .expect("invalid address"),
        );

        let client = ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .redirect(Policy::default())
            .build()
            .expect("can't build client");

        Self { addr, client }
    }

    async fn get<T: DeserializeOwned>(
        client: reqwest::Client,
        addr: Arc<Url>,
        path: &str,
    ) -> anyhow::Result<T> {
        let uri = addr.join(path).context("build url")?;
        let res = client.get(uri).send().await.context("get")?;

        res.json().await.context("unmarshal json")
    }

    fn search_path(filter: &StationsFilter) -> String {
        let mut path = "/json/stations/search?hidebroken=true&bitrateMin=320".to_string();

        if let Some(limit) = filter.limit {
            path.push_str("&limit=");
            path.push_str(&limit.to_string());
        }

        if let Some(offset) = filter.offset {
            path.push_str("&offset=");
            path.push_str(&offset.to_string());
        }

        if let Some(order_by) = filter.order_by.as_ref().map(Into::into) {
            path.push_str("&order=");
            path.push_str(order_by);
        }

        path
    }
}

impl Client for RadioBrowser {
    fn name(&self) -> &str {
        RADIO_BROWSER_NAME
    }

    fn search(&self, filter: &StationsFilter) -> BoxFuture<anyhow::Result<Vec<Station>>> {
        let addr = self.addr.clone();
        let path = Self::search_path(filter);
        let client = self.client.clone();

        Box::pin(async move {
            let supported_codecs = ["MP3", "FLAC"];
            let stations: Vec<RadioStation> = Self::get(client, addr, &path).await?;

            Ok(stations
                .into_iter()
                .filter(|s| supported_codecs.contains(&s.codec.as_str()))
                .map(Station::from)
                .collect())
        })
    }
}

#[derive(Debug, Deserialize)]
struct RadioStation {
    #[serde(rename = "stationuuid")]
    pub uuid: String,
    pub name: String,
    pub url: String,
    pub codec: String,
    pub bitrate: u32,
    pub tags: String,
    pub country: String,
}

impl From<RadioStation> for Station {
    fn from(value: RadioStation) -> Self {
        Self {
            id: 0,
            provider: RADIO_BROWSER_NAME.to_string(),
            provider_id: value.uuid,
            name: value.name,
            url: value.url,
            codec: value.codec,
            bitrate: value.bitrate,
            tags: value.tags.into(),
            country: value.country,
        }
    }
}

impl From<&OrderBy> for &str {
    fn from(value: &OrderBy) -> Self {
        match value {
            OrderBy::CreatedAt => "",
        }
    }
}

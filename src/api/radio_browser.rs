use std::sync::Arc;

use anyhow::Context;
use futures::future::BoxFuture;
use reqwest::{redirect::Policy, ClientBuilder, Url};
use serde::{de::DeserializeOwned, Deserialize};

use crate::models::Station;

use super::Client;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct RadioBrowser {
    addr: Arc<Url>,
    client: reqwest::Client,
}

impl RadioBrowser {
    pub fn new(addr: &str) -> Self {
        let addr = Arc::new(addr.to_string().parse().unwrap()); // todo: expect or error.
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
}

impl Client for RadioBrowser {
    fn stations(&self) -> BoxFuture<anyhow::Result<Vec<Station>>> {
        let addr = self.addr.clone();
        let path = "/json/stations/search?hidebroken=true&min&bitrateMin=320&order=clickcount";
        let client = self.client.clone();

        Box::pin(async move {
            let supported_codecs = ["MP3", "FLAC"];
            let stations: Vec<RadioStation> = Self::get(client, addr, path).await?;

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
            external_id: Some(value.uuid),
            name: value.name,
            url: value.url,
            codec: value.codec,
            bitrate: value.bitrate,
            tags: value.tags.into(),
            country: value.country,
        }
    }
}

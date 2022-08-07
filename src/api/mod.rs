use anyhow::Context;
use reqwest::{redirect::Policy, ClientBuilder, Url};
use serde::de::DeserializeOwned;
use std::collections::HashSet;

use crate::models::Station;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct Client {
    addr: Url,
    client: reqwest::Client,
}

impl Client {
    pub fn new(addr: &str) -> Self {
        let addr = addr.to_string().parse().unwrap(); // todo: expect or error.
        let client = ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .redirect(Policy::default())
            .build()
            .expect("can't build client");

        Self { addr, client }
    }

    pub async fn stations(&self) -> anyhow::Result<Vec<Station>> {
        let supported_codecs = HashSet::from(["MP3", "FLAC"]);

        let result: Vec<Station> = self
            .get("/json/stations/search?hidebroken=true&min&bitrateMin=320&order=clickcount")
            .await?;

        Ok(result
            .into_iter()
            .filter(|s| supported_codecs.contains(&s.codec.as_str()))
            .collect())
    }

    async fn get<T: DeserializeOwned>(&self, uri: &str) -> anyhow::Result<T> {
        let uri = self.addr.join(uri).context("build uri")?;
        let res = self.client.get(uri).send().await.context("get")?;

        res.json().await.context("unmarshal json")
    }
}

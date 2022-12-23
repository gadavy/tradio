use reqwest::{redirect::Policy, ClientBuilder, Url};
use serde::Deserialize;

use crate::models::{OrderBy, Station, StationsFilter};

use super::Client;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const PROVIDER_NAME: &str = "radio-browser";

#[derive(Debug, Clone)]
pub struct RadioBrowser {
    addr: Url,
    client: reqwest::Client,
}

impl RadioBrowser {
    pub fn new() -> Self {
        let addr = "https://de1.api.radio-browser.info"
            .parse()
            .expect("invalid address");

        let client = ClientBuilder::new()
            .user_agent(APP_USER_AGENT)
            .redirect(Policy::default())
            .build()
            .expect("can't build client");

        Self { addr, client }
    }

    fn search_url(&self, filter: &StationsFilter) -> Url {
        let mut url = self.addr.clone();
        url.set_path("/json/stations/search");
        url.query_pairs_mut().append_pair("hidebroken", "true");

        if let Some(limit) = filter.limit {
            url.query_pairs_mut()
                .append_pair("limit", &limit.to_string().as_str());
        }

        if let Some(offset) = filter.offset {
            url.query_pairs_mut()
                .append_pair("offset", &offset.to_string().as_str());
        }

        if let Some(order_by) = filter.order_by.as_ref() {
            url.query_pairs_mut().append_pair("order", order_by.into());
        };

        url
    }
}

impl Client for RadioBrowser {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    async fn search(&self, filter: &StationsFilter) -> anyhow::Result<Vec<Station>> {
        let url = self.search_url(filter);
        let resp = self.client.get(url).send().await?;
        let data = resp.json::<Vec<RadioStation>>().await?;

        let codecs = ["MP3", "FLAC"];

        Ok(data
            .into_iter()
            .filter(|s| codecs.contains(&s.codec.as_str()))
            .map(Station::from)
            .collect())
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
            provider: PROVIDER_NAME.to_string(),
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

#[cfg(test)]
mod tests {
    use super::{OrderBy, RadioBrowser, StationsFilter};

    #[test]
    fn test_search_url() {
        let rb = RadioBrowser::new();
        let test_data = [
            (
                StationsFilter {
                    order_by: None,
                    limit: None,
                    offset: None,
                },
                "hidebroken=true",
            ),
            (
                StationsFilter {
                    order_by: Some(OrderBy::CreatedAt),
                    limit: None,
                    offset: None,
                },
                "hidebroken=true&order=",
            ),
            (
                StationsFilter {
                    order_by: None,
                    limit: Some(10),
                    offset: None,
                },
                "hidebroken=true&limit=10",
            ),
            (
                StationsFilter {
                    order_by: None,
                    limit: None,
                    offset: Some(20),
                },
                "hidebroken=true&offset=20",
            ),
        ];

        for (filter, want) in test_data {
            assert_eq!(rb.search_url(&filter).query(), Some(want));
        }
    }
}

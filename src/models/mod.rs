use serde::{Deserialize, Deserializer};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Deserialize, Debug)]
pub struct Station {
    #[serde(rename = "stationuuid")]
    pub uuid: String,
    pub name: String,
    pub url: String,
    pub codec: String,
    pub bitrate: u32,
    pub tags: Tags,
    pub country: String,
}

#[derive(Clone, Debug)]
pub struct Tags(Vec<String>);

impl<'de> Deserialize<'de> for Tags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(s.split(',').map(|p| p.trim().to_string()).collect()))
    }
}

impl Deref for Tags {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Tags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Station {
    pub id: i64,
    pub provider: String,
    pub provider_id: String,
    pub name: String,
    pub url: String,
    pub codec: String,
    pub bitrate: u32,
    pub tags: Tags,
    pub country: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tags(Vec<String>);

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

impl ToString for Tags {
    fn to_string(&self) -> String {
        self.join(",")
    }
}

impl From<String> for Tags {
    fn from(value: String) -> Self {
        Self(value.split(',').map(|p| p.trim().to_string()).collect())
    }
}

impl From<&str> for Tags {
    fn from(value: &str) -> Self {
        Self(value.split(',').map(|p| p.trim().to_string()).collect())
    }
}

impl From<Vec<String>> for Tags {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

pub enum OrderBy {
    CreatedAt,
}

#[derive(Default)]
pub struct StationsFilter {
    pub order_by: Option<OrderBy>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

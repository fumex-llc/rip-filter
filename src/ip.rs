use dashmap::{DashMap, DashSet};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::IpAddr, sync::Arc};

pub(crate) const PROXY_PROTOCOL_HEADER_LEN: usize = 108;

pub(crate) type IpSet = Arc<DashSet<IpAddr>>;
pub(crate) type IpMap = Arc<DashMap<IpAddr, IpKind>>;
pub(crate) type IpRangeSet = Arc<Vec<IpNet>>;

#[derive(Copy, Clone, Debug, Deserialize, Default, Serialize)]
pub(crate) enum IpKind {
    Residential,
    Business,
    Wireless,
    Hosting,
    #[default]
    #[serde(other)]
    #[serde(rename = "null")]
    Null,
}

impl IpKind {
    pub fn is_allowed(&self) -> bool {
        use IpKind::*;
        !matches!(self, Residential | Business | Hosting)
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub(crate) struct ProxyCheckResponse {
    pub status: String,
    #[serde(flatten)]
    ips: HashMap<String, IpInfo>,
}

impl ProxyCheckResponse {
    pub fn ip_kind(self) -> IpKind {
        self.ips.into_iter().next().unwrap_or_default().1.r#type
    }
}

#[derive(Deserialize, Default, Debug, Serialize)]
struct IpInfo {
    #[serde(rename = "type")]
    r#type: IpKind,
}

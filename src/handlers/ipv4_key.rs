use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Query {
    pub(super) from: Ipv4Addr,
    pub(super) to: Ipv4Addr,
}

impl Query {
    pub(super) fn octets(&self) -> ([u8; 4], [u8; 4]) {
        let Self { from, to } = self;
        (from.octets(), to.octets())
    }
}

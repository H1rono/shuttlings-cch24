use std::net::Ipv6Addr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Query {
    pub(super) from: Ipv6Addr,
    pub(super) key: Ipv6Addr,
}

impl Query {
    pub(super) fn to_bits(self) -> (u128, u128) {
        let Self { from, key } = self;
        (from.to_bits(), key.to_bits())
    }
}

use std::net::Ipv6Addr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Query {
    pub(super) from: Ipv6Addr,
    pub(super) to: Ipv6Addr,
}

impl Query {
    pub(super) fn to_bits(self) -> (u128, u128) {
        let Self { from, to } = self;
        (from.to_bits(), to.to_bits())
    }
}

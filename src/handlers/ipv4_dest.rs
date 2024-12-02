use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Query {
    pub(super) from: Ipv4Addr,
    pub(super) key: Ipv4Addr,
}

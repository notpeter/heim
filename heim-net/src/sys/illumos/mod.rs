use std::net::{Ipv4Addr, SocketAddrV4};

use heim_common::prelude::*;
use heim_common::units::{information, Information};

use crate::Address;

#[derive(Debug, Default)]
pub struct Nic;

impl Nic {
    pub fn name(&self) -> &str {
        ""
    }

    pub fn index(&self) -> Option<u32> {
        None
    }

    pub fn address(&self) -> Address {
        Address::Inet(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
    }

    pub fn netmask(&self) -> Option<Address> {
        None
    }

    pub fn destination(&self) -> Option<Address> {
        None
    }

    pub fn is_up(&self) -> bool {
        false
    }

    pub fn is_running(&self) -> bool {
        false
    }

    pub fn is_loopback(&self) -> bool {
        false
    }

    pub fn is_multicast(&self) -> bool {
        false
    }
}

#[derive(Debug, Default)]
pub struct IoCounters;

impl IoCounters {
    pub fn interface(&self) -> &str {
        ""
    }

    pub fn bytes_sent(&self) -> Information {
        Information::new::<information::byte>(0)
    }

    pub fn bytes_recv(&self) -> Information {
        Information::new::<information::byte>(0)
    }

    pub fn packets_sent(&self) -> u64 {
        0
    }

    pub fn packets_recv(&self) -> u64 {
        0
    }

    pub fn errors_sent(&self) -> u64 {
        0
    }

    pub fn errors_recv(&self) -> u64 {
        0
    }

    pub fn drop_recv(&self) -> u64 {
        0
    }
}

pub async fn io_counters() -> Result<impl Stream<Item = Result<IoCounters>>> {
    Ok(stream::empty())
}

pub async fn nic() -> Result<impl Stream<Item = Result<Nic>> + Send + Sync> {
    Ok(stream::empty())
}

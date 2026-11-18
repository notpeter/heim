use macaddr::MacAddr;
use nix::ifaddrs;
use nix::net::if_::if_nametoindex;
use nix::net::if_::InterfaceFlags;
use nix::sys::socket;

use heim_common::prelude::*;

use crate::Address;

#[derive(Debug)]
pub struct Nic(ifaddrs::InterfaceAddress);

impl Nic {
    pub fn name(&self) -> &str {
        self.0.interface_name.as_str()
    }

    pub fn index(&self) -> Option<u32> {
        if_nametoindex(self.name()).ok()
    }

    pub fn address(&self) -> Address {
        self.0
            .address
            .as_ref()
            .and_then(address)
            .expect("NIC stream should exclude entries without supported addresses")
    }

    pub fn netmask(&self) -> Option<Address> {
        self.0.netmask.as_ref().and_then(address)
    }

    pub fn broadcast(&self) -> Option<Address> {
        self.0.broadcast.as_ref().and_then(address)
    }

    pub fn destination(&self) -> Option<Address> {
        self.0.destination.as_ref().and_then(address)
    }

    pub fn is_up(&self) -> bool {
        self.0.flags.contains(InterfaceFlags::IFF_UP)
    }

    pub fn is_running(&self) -> bool {
        self.0.flags.contains(InterfaceFlags::IFF_RUNNING)
    }

    pub fn is_broadcast(&self) -> bool {
        self.0.flags.contains(InterfaceFlags::IFF_BROADCAST)
    }

    pub fn is_loopback(&self) -> bool {
        self.0.flags.contains(InterfaceFlags::IFF_LOOPBACK)
    }

    pub fn is_point_to_point(&self) -> bool {
        self.0.flags.contains(InterfaceFlags::IFF_POINTOPOINT)
    }

    pub fn is_multicast(&self) -> bool {
        self.0.flags.contains(InterfaceFlags::IFF_MULTICAST)
    }
}

pub async fn nic() -> Result<impl Stream<Item = Result<Nic>> + Send + Sync> {
    let iter = ifaddrs::getifaddrs()?
        .filter_map(|addr| {
            if addr.address.as_ref().and_then(address).is_some() {
                Some(Ok(Nic(addr)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(stream::iter(iter))
}

fn address(s: &socket::SockaddrStorage) -> Option<Address> {
    if let Some(addr) = s.as_sockaddr_in() {
        Some(Address::Inet((*addr).into()))
    } else if let Some(addr) = s.as_sockaddr_in6() {
        Some(Address::Inet6((*addr).into()))
    } else {
        s.as_link_addr()
            .and_then(|addr| addr.addr())
            .map(|mac| Address::Link(MacAddr::from(mac)))
    }
}

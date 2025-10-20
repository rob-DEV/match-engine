use std::net::Ipv4Addr;

pub const MAX_UDP_PACKET_SIZE: usize = 64;
pub const MULTICAST_INTERFACE: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

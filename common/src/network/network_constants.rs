use std::net::Ipv4Addr;

pub const MAX_UDP_PACKET_SIZE: usize = 2048;
pub const LOOPBACK_INTERFACE: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
pub const DOCKER_MULTICAST_INTERFACE: Ipv4Addr = Ipv4Addr::new(172, 17, 0, 1);
pub const ROUTER_INTERFACE: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 108);
pub const MULTICAST_INTERFACE: Ipv4Addr = LOOPBACK_INTERFACE;

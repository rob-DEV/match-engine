use crate::network::network_constants::MULTICAST_INTERFACE;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

pub fn multicast_receiver(port: u16) -> UdpSocket {
    let group = Ipv4Addr::new(239, 255, 0, 1);

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.set_reuse_port(true).unwrap();

    let bind_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    socket.bind(&bind_addr.into()).unwrap();

    socket
        .join_multicast_v4(&group, &MULTICAST_INTERFACE)
        .unwrap();

    std::net::UdpSocket::from(socket)
}

pub fn multicast_sender() -> UdpSocket {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.set_reuse_port(true).unwrap();

    socket
        .bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into())
        .unwrap();

    socket.set_multicast_if_v4(&MULTICAST_INTERFACE).unwrap();

    socket.set_multicast_ttl_v4(1).unwrap();

    std::net::UdpSocket::from(socket)
}

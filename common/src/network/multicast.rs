use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

pub fn multicast_udp_socket(port: u16, bind: bool) -> UdpSocket {
    use socket2::{Domain, Protocol, Socket, Type};
    let udp_multicast_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    udp_multicast_socket.set_reuse_address(true).expect("failed to set reuse address");
    udp_multicast_socket.set_reuse_port(true).expect("failed to set reuse port");
    udp_multicast_socket.broadcast().expect("failed to broadcast UDP socket");

    if bind {
        udp_multicast_socket.bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port).into()).unwrap();
    }

    std::net::UdpSocket::from(udp_multicast_socket)
}
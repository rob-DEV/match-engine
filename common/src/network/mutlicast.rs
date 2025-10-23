use crate::network::network_constants::MULTICAST_INTERFACE;
use libc::{setsockopt, SOL_SOCKET, SO_BUSY_POLL, SO_PRIORITY};
use nix::libc;
use nix::sys::socket::{sendmsg, MsgFlags, SockaddrIn};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::IoSlice;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::os::fd::AsRawFd;
use std::{io, mem};

pub fn multicast_receiver(port: u16) -> UdpSocket {
    let group = Ipv4Addr::new(239, 255, 0, 1);

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.set_reuse_port(true).unwrap();

    set_socket_priority_busy_spin(&socket);

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

    set_socket_priority_busy_spin(&socket);

    socket
        .bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into())
        .unwrap();

    socket.set_multicast_if_v4(&MULTICAST_INTERFACE).unwrap();

    socket.set_multicast_ttl_v4(1).unwrap();

    std::net::UdpSocket::from(socket)
}

fn set_socket_priority_busy_spin(socket: &Socket) {
    let fd = socket.as_raw_fd();

    let busy_poll: i32 = 10; // microseconds
    let ret = unsafe {
        setsockopt(
            fd,
            SOL_SOCKET,
            SO_BUSY_POLL,
            &busy_poll as *const _ as *const libc::c_void,
            mem::size_of_val(&busy_poll) as u32,
        )
    };

    if ret != 0 {
        eprintln!("Failed to set SO_BUSY_POLL: {}", io::Error::last_os_error());
    }

    let prio: i32 = 6; // 0–6 (higher = higher priority)
    let ret = unsafe {
        setsockopt(
            fd,
            SOL_SOCKET,
            SO_PRIORITY,
            &prio as *const _ as *const libc::c_void,
            mem::size_of_val(&prio) as u32,
        )
    };
    if ret != 0 {
        eprintln!("Failed to set SO_PRIORITY: {}", io::Error::last_os_error());
    }
}

pub fn send_udp_msg(socket: &Socket, data: &[u8]) {
    // Create an IoSlice for zero-copy pointer to your data

    // Convert std::net::SocketAddrV4 to nix::SockAddr

    let target = SocketAddrV4::new(Ipv4Addr::new(239, 255, 0, 1), 3000);
    let dest = SockaddrIn::from(target);

    // Prepare the data buffer
    let data = [0xABu8; 128];
    let iov = [IoSlice::new(&data)];

    let sent = sendmsg(
        socket.as_raw_fd(),
        &iov,
        &[], // no control messages
        MsgFlags::empty(),
        Some(&dest),
    )
    .unwrap();
}

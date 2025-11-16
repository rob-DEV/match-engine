use crate::util::time::system_nanos;
use bitcode::Buffer;
use libc::{iovec, mmsghdr, msghdr, recvmmsg, sendmmsg, MSG_DONTWAIT};
use std::net::{SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::{mem, ptr};

const BATCH_SIZE: usize = 32;
const MAX_PACKET_SIZE: usize = 2048;

pub struct MulticastBatchedSocket {
    udp_socket: Arc<UdpSocket>,
    sock_addr: SocketAddr,

    // send batching
    batch: Vec<Vec<u8>>,
    last_flush_ns: u64,

    // pre-built recv/send descriptors
    mmsgs: [mmsghdr; BATCH_SIZE],
    iovecs: [iovec; BATCH_SIZE],
    bufs: [[u8; MAX_PACKET_SIZE]; BATCH_SIZE],
}

impl MulticastBatchedSocket {
    pub fn new(udp_socket: Arc<UdpSocket>, sock_addr: SocketAddr) -> Self {
        let mut me = MulticastBatchedSocket {
            udp_socket,
            sock_addr,
            batch: Vec::with_capacity(BATCH_SIZE),
            last_flush_ns: system_nanos(),
            mmsgs: unsafe { mem::zeroed() },
            iovecs: unsafe { mem::zeroed() },
            bufs: [[0u8; MAX_PACKET_SIZE]; BATCH_SIZE],
        };

        // Build static recv descriptors ONCE
        for i in 0..BATCH_SIZE {
            me.iovecs[i] = iovec {
                iov_base: me.bufs[i].as_mut_ptr() as *mut _,
                iov_len: MAX_PACKET_SIZE,
            };

            me.mmsgs[i].msg_hdr = msghdr {
                msg_name: ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: &mut me.iovecs[i] as *mut _,
                msg_iovlen: 1,
                msg_control: ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            };

            me.mmsgs[i].msg_len = 0;
        }

        me
    }

    pub fn send(&mut self, packet: Vec<u8>) {
        self.batch.push(packet);
        if self.batch.len() == BATCH_SIZE {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if self.batch.is_empty() {
            return;
        }

        for i in 0..self.batch.len() {
            let pkt = &self.batch[i];

            self.iovecs[i] = iovec {
                iov_base: pkt.as_ptr() as *mut _,
                iov_len: pkt.len(),
            };

            self.mmsgs[i].msg_hdr = msghdr {
                msg_name: &self.sock_addr as *const _ as *mut _,
                msg_namelen: mem::size_of::<libc::sockaddr_in>() as u32,
                msg_iov: &mut self.iovecs[i] as *mut _,
                msg_iovlen: 1,
                msg_control: ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            };
        }

        unsafe {
            sendmmsg(
                self.udp_socket.as_raw_fd(),
                self.mmsgs.as_mut_ptr(),
                self.batch.len() as u32,
                0,
            );
        }

        self.batch.clear();
        self.last_flush_ns = system_nanos();
    }

    pub fn recv(&mut self) -> &[[u8; MAX_PACKET_SIZE]] {
        let n = unsafe {
            recvmmsg(
                self.udp_socket.as_raw_fd(),
                self.mmsgs.as_mut_ptr(),
                BATCH_SIZE as u32,
                MSG_DONTWAIT,
                ptr::null_mut(),
            )
        };

        if n <= 0 {
            return &[];
        }

        &self.bufs[..n as usize]
    }
}

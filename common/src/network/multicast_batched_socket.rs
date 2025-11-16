use crate::memory::memory::uninitialized_arr;
use crate::util::time::system_nanos;
use bitcode::Buffer;
use libc::{iovec, mmsghdr, msghdr, sendmmsg};
use std::net::{SocketAddr, UdpSocket};
use std::os::fd::AsRawFd;
use std::sync::Arc;

const BATCH_SIZE: usize = 32;
const MAX_PACKET_SIZE: usize = 1024;
const FLUSH_INTERVAL_NS: u64 = 10_000;

pub struct MulticastBatchedSocket {
    udp_socket: Arc<UdpSocket>,
    sock_addr: SocketAddr,
    batch: Vec<Vec<u8>>,
    last_flush_ns: u64,
    encoding_buffer: Buffer,
    mmsgs: [mmsghdr; BATCH_SIZE],
    iovecs: [iovec; BATCH_SIZE],
}

impl MulticastBatchedSocket {
    pub fn new(udp_socket: Arc<UdpSocket>, sock_addr: SocketAddr) -> Self {
        MulticastBatchedSocket {
            udp_socket,
            sock_addr,
            batch: Vec::with_capacity(BATCH_SIZE),
            last_flush_ns: system_nanos(),
            encoding_buffer: Buffer::new(),
            mmsgs: uninitialized_arr(),
            iovecs: uninitialized_arr(),
        }
    }

    pub fn send(&mut self, packet: Vec<u8>) {
        self.batch.push(packet);

        if self.batch.len() == BATCH_SIZE {
            self.flush();
            self.last_flush_ns = system_nanos();
        }
    }

    pub fn flush(&mut self) {
        if self.batch.is_empty() {
            return;
        }

        let mut idx = 0;
        for pkt in &self.batch {
            let iov = iovec {
                iov_base: pkt.as_ptr() as *mut _,
                iov_len: pkt.len(),
            };

            let mut hdr: msghdr = unsafe { std::mem::zeroed() };
            hdr.msg_name = &self.sock_addr as *const _ as *mut _;
            hdr.msg_namelen = std::mem::size_of::<libc::sockaddr_in>() as u32;
            hdr.msg_iov = &iov as *const _ as *mut _;
            hdr.msg_iovlen = 1;

            self.iovecs[idx] = iov;
            self.mmsgs[idx] = mmsghdr {
                msg_hdr: hdr,
                msg_len: 0,
            };

            idx += 1;
        }

        unsafe {
            let _ = sendmmsg(
                self.udp_socket.as_raw_fd(),
                self.mmsgs.as_mut_ptr(),
                self.mmsgs.len() as u32,
                0,
            );
        }
    }
}

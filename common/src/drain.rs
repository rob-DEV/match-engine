use crate::util::time::system_nanos;
use std::sync::mpsc::Receiver;

pub fn rx_drain_with_timeout<T, U>(rx: &Receiver<T>, pre_allocated_buffer: &mut Vec<U>, mapper: fn(T) -> U, timeout_nanos: u64) -> usize {
    let nanos = system_nanos();
    while let Ok(inbound_engine_message) = rx.recv_timeout(std::time::Duration::from_nanos(timeout_nanos)) {
        let t = system_nanos() - nanos;
        pre_allocated_buffer.push(mapper(inbound_engine_message));

        if pre_allocated_buffer.len() == pre_allocated_buffer.capacity() || t >= timeout_nanos {
            return pre_allocated_buffer.len();
        }
    }

    pre_allocated_buffer.len()
}
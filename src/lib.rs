pub mod transport;

pub use transport::{
    FleetMsgHeader, MessageType, MulticastSender, start_multicast_rx
};

use std::net::Ipv4Addr;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

/// Example function showing how to use the multicast transport
/// Note: This is just a demonstration - in practice you'd use async_std::main
/// or integrate with your preferred async runtime
pub async fn run_example() -> std::io::Result<()> {
    let group = Ipv4Addr::new(239, 1, 1, 1);
    let port = 12345;

    // Example message handler
    let handler = |header: FleetMsgHeader, payload: Vec<u8>, addr: std::net::SocketAddr| {
        println!("Received {:?} from {}: {} bytes",
                 header.message_type(), addr, payload.len());
    };

    // Start multicast receiver (this would run indefinitely)
    start_multicast_rx(group, port, handler).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

use async_std::net::{UdpSocket, SocketAddr};
use zerocopy::{AsBytes, FromBytes, FromZeroes};
use std::net::{Ipv4Addr, IpAddr};
use std::time::{SystemTime, UNIX_EPOCH};

/// Fleet message types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
    Heartbeat = 1,
    Data = 2,
    Control = 3,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => MessageType::Heartbeat,
            2 => MessageType::Data,
            3 => MessageType::Control,
            _ => MessageType::Heartbeat, // Default fallback
        }
    }
}

/// Fleet message header with proper fields
#[repr(C)]
#[derive(FromBytes, AsBytes, FromZeroes, Debug, Clone, Copy)]
pub struct FleetMsgHeader {
    pub magic: u32,        // Magic number for validation (0xFEED)
    pub version: u8,       // Protocol version
    pub msg_type: u8,      // Message type (see MessageType enum)
    pub sequence: u16,     // Sequence number
    pub timestamp: u64,    // Unix timestamp in milliseconds
    pub sender_id: u32,    // Unique sender identifier
    pub payload_len: u16,  // Length of payload following header
    pub checksum: u16,     // Simple checksum for integrity
}

impl FleetMsgHeader {
    const MAGIC: u32 = 0xFEED;
    const VERSION: u8 = 1;

    pub fn new(msg_type: MessageType, sender_id: u32, sequence: u16, payload_len: u16) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut header = Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            msg_type: msg_type as u8,
            sequence,
            timestamp,
            sender_id,
            payload_len,
            checksum: 0,
        };

        // Calculate simple checksum (sum of all bytes except checksum field)
        header.checksum = header.calculate_checksum();
        header
    }

    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC &&
        self.version == Self::VERSION &&
        self.checksum == self.calculate_checksum_without_field()
    }

    fn calculate_checksum(&self) -> u16 {
        let bytes = self.as_bytes();
        let mut sum: u32 = 0;

        // Sum all bytes except the checksum field (last 2 bytes)
        for &byte in &bytes[..bytes.len() - 2] {
            sum += byte as u32;
        }

        (sum & 0xFFFF) as u16
    }

    fn calculate_checksum_without_field(&self) -> u16 {
        let mut temp = self.clone();
        temp.checksum = 0;
        temp.calculate_checksum()
    }

    pub fn message_type(&self) -> MessageType {
        MessageType::from(self.msg_type)
    }
}

/// Multicast receiver that processes incoming fleet messages
pub async fn start_multicast_rx(
    group: Ipv4Addr,
    port: u16,
    mut message_handler: impl FnMut(FleetMsgHeader, Vec<u8>, SocketAddr) + Send + 'static
) -> std::io::Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", port)).await?;
    socket.join_multicast_v4(group, Ipv4Addr::UNSPECIFIED)?;

    println!("Started multicast receiver on {}:{}", group, port);

    let mut buf = vec![0u8; 1500]; // Standard MTU size

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                if len < std::mem::size_of::<FleetMsgHeader>() {
                    eprintln!("Received packet too small for header from {}", addr);
                    continue;
                }

                if let Some(header) = FleetMsgHeader::read_from_prefix(&buf[..len]) {
                    if header.is_valid() {
                        let header_size = std::mem::size_of::<FleetMsgHeader>();
                        let payload = if len > header_size {
                            buf[header_size..len].to_vec()
                        } else {
                            Vec::new()
                        };

                        // Verify payload length matches header
                        if payload.len() == header.payload_len as usize {
                            message_handler(header.clone(), payload, addr);
                        } else {
                            eprintln!("Payload length mismatch from {}: expected {}, got {}",
                                     addr, header.payload_len, payload.len());
                        }
                    } else {
                        eprintln!("Invalid message header from {}", addr);
                    }
                } else {
                    eprintln!("Failed to parse message header from {}", addr);
                }
            }
            Err(e) => {
                eprintln!("Error receiving multicast message: {}", e);
                // Continue listening despite errors
            }
        }
    }
}

/// Multicast sender for broadcasting fleet messages
pub struct MulticastSender {
    socket: UdpSocket,
    group: Ipv4Addr,
    port: u16,
    sender_id: u32,
    sequence: u16,
}

impl MulticastSender {
    pub async fn new(group: Ipv4Addr, port: u16, sender_id: u32) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_multicast_ttl_v4(1)?; // Local network only

        println!("Created multicast sender for {}:{} with ID {}", group, port, sender_id);

        Ok(Self {
            socket,
            group,
            port,
            sender_id,
            sequence: 0,
        })
    }

    pub async fn send_message(
        &mut self,
        msg_type: MessageType,
        payload: &[u8]
    ) -> std::io::Result<()> {
        let header = FleetMsgHeader::new(
            msg_type,
            self.sender_id,
            self.sequence,
            payload.len() as u16
        );

        self.sequence = self.sequence.wrapping_add(1);

        let mut message = Vec::new();
        message.extend_from_slice(header.as_bytes());
        message.extend_from_slice(payload);

        let addr = SocketAddr::new(IpAddr::V4(self.group), self.port);
        self.socket.send_to(&message, addr).await?;

        println!("Sent {} message (seq: {}, {} bytes payload)",
                 format!("{:?}", msg_type), header.sequence, payload.len());

        Ok(())
    }

    pub async fn send_heartbeat(&mut self) -> std::io::Result<()> {
        self.send_message(MessageType::Heartbeat, b"").await
    }

    pub async fn send_data(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.send_message(MessageType::Data, data).await
    }

    pub async fn send_control(&mut self, command: &str) -> std::io::Result<()> {
        self.send_message(MessageType::Control, command.as_bytes()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[async_std::test]
    async fn test_header_creation_and_validation() {
        let header = FleetMsgHeader::new(MessageType::Data, 12345, 100, 256);

        assert_eq!(header.magic, 0xFEED);
        assert_eq!(header.version, 1);
        assert_eq!(header.msg_type, MessageType::Data as u8);
        assert_eq!(header.sender_id, 12345);
        assert_eq!(header.sequence, 100);
        assert_eq!(header.payload_len, 256);
        assert!(header.is_valid());
        assert_eq!(header.message_type(), MessageType::Data);
    }

    #[async_std::test]
    async fn test_header_serialization() {
        let original = FleetMsgHeader::new(MessageType::Heartbeat, 54321, 200, 0);
        let bytes = original.as_bytes();

        let deserialized = FleetMsgHeader::read_from_prefix(bytes).unwrap();

        assert_eq!(original.magic, deserialized.magic);
        assert_eq!(original.sender_id, deserialized.sender_id);
        assert_eq!(original.sequence, deserialized.sequence);
        assert!(deserialized.is_valid());
    }

    #[async_std::test]
    async fn test_multicast_send_receive() {
        let group = Ipv4Addr::new(239, 1, 1, 1);
        let port = 12345;
        let sender_id = 999;

        // Shared state to capture received messages
        let received_messages = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received_messages.clone();

        // Start receiver in background
        let receiver_task = task::spawn(async move {
            let handler = move |header: FleetMsgHeader, payload: Vec<u8>, _addr: SocketAddr| {
                received_clone.lock().unwrap().push((header, payload));
            };

            // Run receiver for a short time
            let receiver_future = start_multicast_rx(group, port, handler);
            let timeout_future = task::sleep(Duration::from_millis(500));

            // Race between receiver and timeout
            futures::future::select(
                Box::pin(receiver_future),
                Box::pin(timeout_future)
            ).await;
        });

        // Give receiver time to start
        task::sleep(Duration::from_millis(100)).await;

        // Create sender and send test messages
        let mut sender = MulticastSender::new(group, port, sender_id).await.unwrap();

        sender.send_heartbeat().await.unwrap();
        sender.send_data(b"test data").await.unwrap();
        sender.send_control("test command").await.unwrap();

        // Wait a bit for messages to be received
        task::sleep(Duration::from_millis(200)).await;

        // Stop receiver
        receiver_task.cancel().await;

        // Check received messages
        let messages = received_messages.lock().unwrap();
        assert!(messages.len() >= 1, "Should have received at least one message");

        // Verify message types and content
        for (header, payload) in messages.iter() {
            assert_eq!(header.sender_id, sender_id);
            assert!(header.is_valid());

            match header.message_type() {
                MessageType::Heartbeat => assert_eq!(payload.len(), 0),
                MessageType::Data => assert_eq!(payload, b"test data"),
                MessageType::Control => assert_eq!(payload, b"test command"),
            }
        }
    }
}

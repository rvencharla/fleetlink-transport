use fleetlink_transport::{MulticastSender, MessageType, start_multicast_rx, FleetMsgHeader};
use zerocopy::AsBytes;
use async_std::task;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[async_std::test]
async fn test_multicast_communication() {
    let group = Ipv4Addr::new(239, 1, 1, 2); // Different group to avoid conflicts
    let port = 12346;
    let sender_id = 12345;
    
    // Shared state to capture received messages
    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received_messages.clone();
    
    // Start receiver in background
    let receiver_task = task::spawn(async move {
        let handler = move |header: FleetMsgHeader, payload: Vec<u8>, addr: SocketAddr| {
            println!("Test received: {:?} from {} with {} bytes", 
                     header.message_type(), addr, payload.len());
            received_clone.lock().unwrap().push((header, payload, addr));
        };
        
        // Run receiver for test duration
        let receiver_future = start_multicast_rx(group, port, handler);
        let timeout_future = task::sleep(Duration::from_secs(3));
        
        futures::future::select(
            Box::pin(receiver_future),
            Box::pin(timeout_future)
        ).await;
    });
    
    // Give receiver time to start and bind
    task::sleep(Duration::from_millis(200)).await;
    
    // Create sender and send test messages
    let mut sender = MulticastSender::new(group, port, sender_id).await
        .expect("Failed to create multicast sender");
    
    // Send various message types
    sender.send_heartbeat().await.expect("Failed to send heartbeat");
    task::sleep(Duration::from_millis(100)).await;
    
    sender.send_data(b"Hello, Fleet!").await.expect("Failed to send data");
    task::sleep(Duration::from_millis(100)).await;
    
    sender.send_control("SHUTDOWN").await.expect("Failed to send control");
    task::sleep(Duration::from_millis(100)).await;
    
    // Send a few more messages to test sequence numbers
    for i in 0..3 {
        let data = format!("Message {}", i);
        sender.send_data(data.as_bytes()).await.expect("Failed to send data");
        task::sleep(Duration::from_millis(50)).await;
    }
    
    // Wait for messages to be processed
    task::sleep(Duration::from_millis(500)).await;
    
    // Stop receiver
    receiver_task.cancel().await;
    
    // Verify received messages
    let messages = received_messages.lock().unwrap();
    println!("Total messages received: {}", messages.len());
    
    assert!(messages.len() >= 5, "Should have received at least 5 messages, got {}", messages.len());
    
    // Check message types and content
    let mut heartbeat_count = 0;
    let mut data_count = 0;
    let mut control_count = 0;
    let mut sequence_numbers = Vec::new();
    
    for (header, payload, _addr) in messages.iter() {
        assert_eq!(header.sender_id, sender_id);
        assert!(header.is_valid(), "Message header should be valid");
        sequence_numbers.push(header.sequence);
        
        match header.message_type() {
            MessageType::Heartbeat => {
                heartbeat_count += 1;
                assert_eq!(payload.len(), 0, "Heartbeat should have empty payload");
            },
            MessageType::Data => {
                data_count += 1;
                assert!(payload.len() > 0, "Data message should have payload");
            },
            MessageType::Control => {
                control_count += 1;
                assert_eq!(payload, b"SHUTDOWN", "Control message should match");
            },
        }
    }
    
    assert!(heartbeat_count >= 1, "Should have received at least 1 heartbeat");
    assert!(data_count >= 4, "Should have received at least 4 data messages");
    assert!(control_count >= 1, "Should have received at least 1 control message");
    
    // Verify sequence numbers are increasing
    for i in 1..sequence_numbers.len() {
        assert!(sequence_numbers[i] > sequence_numbers[i-1], 
                "Sequence numbers should be increasing");
    }
    
    println!("Integration test passed!");
}

#[async_std::test]
async fn test_invalid_message_handling() {
    let group = Ipv4Addr::new(239, 1, 1, 3);
    let port = 12347;
    
    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received_messages.clone();
    
    // Start receiver
    let receiver_task = task::spawn(async move {
        let handler = move |header: FleetMsgHeader, payload: Vec<u8>, _addr: SocketAddr| {
            received_clone.lock().unwrap().push((header, payload));
        };
        
        let receiver_future = start_multicast_rx(group, port, handler);
        let timeout_future = task::sleep(Duration::from_millis(1000));
        
        futures::future::select(
            Box::pin(receiver_future),
            Box::pin(timeout_future)
        ).await;
    });
    
    task::sleep(Duration::from_millis(100)).await;
    
    // Send valid message
    let mut sender = MulticastSender::new(group, port, 999).await.unwrap();
    sender.send_data(b"valid").await.unwrap();
    
    // Try to send invalid data directly (this would be filtered out by the receiver)
    let socket = async_std::net::UdpSocket::bind("0.0.0.0:0").await.unwrap();
    let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(group), port);
    
    // Send too small packet
    socket.send_to(b"tiny", addr).await.unwrap();
    
    // Send packet with invalid magic number
    let mut invalid_header = FleetMsgHeader::new(MessageType::Data, 999, 1, 4);
    invalid_header.magic = 0xDEAD; // Wrong magic
    let mut invalid_message = Vec::new();
    invalid_message.extend_from_slice(invalid_header.as_bytes());
    invalid_message.extend_from_slice(b"test");
    socket.send_to(&invalid_message, addr).await.unwrap();
    
    task::sleep(Duration::from_millis(300)).await;
    receiver_task.cancel().await;
    
    // Should only receive the valid message
    let messages = received_messages.lock().unwrap();
    assert_eq!(messages.len(), 1, "Should only receive valid messages");
    assert_eq!(messages[0].1, b"valid");
}

use fleetlink_transport::{MulticastSender, start_multicast_rx, FleetMsgHeader};
use async_std::task;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("FleetLink Multicast Transport Demo");
    println!("==================================");
    
    let group = Ipv4Addr::new(239, 1, 1, 1);
    let port = 12345;
    
    // Get command line argument to determine mode
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("both");
    
    match mode {
        "sender" => run_sender(group, port).await?,
        "receiver" => run_receiver(group, port).await?,
        "both" => run_both(group, port).await?,
        _ => {
            println!("Usage: {} [sender|receiver|both]", args[0]);
            println!("  sender   - Run only sender");
            println!("  receiver - Run only receiver");
            println!("  both     - Run both sender and receiver (default)");
        }
    }
    
    Ok(())
}

async fn run_sender(group: Ipv4Addr, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting sender mode...");
    
    let mut sender = MulticastSender::new(group, port, 12345).await?;
    
    // Send different types of messages
    for i in 0..10 {
        // Send heartbeat every few iterations
        if i % 3 == 0 {
            sender.send_heartbeat().await?;
        }
        
        // Send data message
        let data = format!("Data message #{}", i);
        sender.send_data(data.as_bytes()).await?;
        
        // Send control message occasionally
        if i % 5 == 0 {
            let command = format!("CONTROL_CMD_{}", i);
            sender.send_control(&command).await?;
        }
        
        task::sleep(Duration::from_millis(1000)).await;
    }
    
    println!("Sender finished");
    Ok(())
}

async fn run_receiver(group: Ipv4Addr, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting receiver mode...");
    println!("Listening for multicast messages on {}:{}...", group, port);
    println!("Press Ctrl+C to stop");
    
    let handler = |header: FleetMsgHeader, payload: Vec<u8>, addr: SocketAddr| {
        let payload_str = String::from_utf8_lossy(&payload);
        println!("[{}] {:?} from {} (seq: {}, {} bytes): {}", 
                 chrono::Utc::now().format("%H:%M:%S%.3f"),
                 header.message_type(), 
                 addr, 
                 header.sequence,
                 payload.len(),
                 payload_str);
    };
    
    start_multicast_rx(group, port, handler).await?;
    Ok(())
}

async fn run_both(group: Ipv4Addr, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting both sender and receiver...");
    
    // Start receiver in background
    let receiver_task = task::spawn(async move {
        let handler = |header: FleetMsgHeader, payload: Vec<u8>, addr: SocketAddr| {
            let payload_str = String::from_utf8_lossy(&payload);
            println!("[RX] {:?} from {} (seq: {}): {}", 
                     header.message_type(), addr, header.sequence, payload_str);
        };
        
        if let Err(e) = start_multicast_rx(group, port, handler).await {
            eprintln!("Receiver error: {}", e);
        }
    });
    
    // Give receiver time to start
    task::sleep(Duration::from_millis(500)).await;
    
    // Start sender
    let mut sender = MulticastSender::new(group, port, 99999).await?;
    
    println!("Sending test messages...");
    
    // Send test messages
    for i in 0..5 {
        sender.send_heartbeat().await?;
        task::sleep(Duration::from_millis(500)).await;
        
        let data = format!("Test data #{}", i);
        sender.send_data(data.as_bytes()).await?;
        task::sleep(Duration::from_millis(500)).await;
        
        if i % 2 == 0 {
            sender.send_control("TEST_COMMAND").await?;
            task::sleep(Duration::from_millis(500)).await;
        }
    }
    
    println!("Demo completed. Receiver will continue running...");
    println!("Press Ctrl+C to stop");
    
    // Keep receiver running
    receiver_task.await;
    
    Ok(())
}

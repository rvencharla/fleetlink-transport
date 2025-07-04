use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use fleetlink_transport::{FleetMsgHeader, MessageType};
use zerocopy::{AsBytes, FromBytes};
use std::time::{Duration, Instant};

// Simulate C-style message handling (inefficient)
#[derive(Debug, Clone)]
struct CStyleMessage {
    magic: u32,
    version: u8,
    msg_type: u8,
    sequence: u16,
    timestamp: u64,
    sender_id: u32,
    payload_len: u16,
    checksum: u16,
    payload: Vec<u8>,
}

impl CStyleMessage {
    fn new(msg_type: u8, sender_id: u32, sequence: u16, payload: Vec<u8>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let mut msg = Self {
            magic: 0xFEED,
            version: 1,
            msg_type,
            sequence,
            timestamp,
            sender_id,
            payload_len: payload.len() as u16,
            checksum: 0,
            payload,
        };
        
        // Inefficient checksum calculation
        msg.checksum = msg.calculate_checksum();
        msg
    }
    
    fn calculate_checksum(&self) -> u16 {
        let mut sum = 0u32;
        sum += self.magic;
        sum += self.version as u32;
        sum += self.msg_type as u32;
        sum += self.sequence as u32;
        sum += (self.timestamp & 0xFFFFFFFF) as u32;
        sum += (self.timestamp >> 32) as u32;
        sum += self.sender_id;
        sum += self.payload_len as u32;
        
        for &byte in &self.payload {
            sum += byte as u32;
        }
        
        (sum & 0xFFFF) as u16
    }
    
    // Simulate C-style serialization (lots of copying)
    fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.magic.to_le_bytes());
        buffer.push(self.version);
        buffer.push(self.msg_type);
        buffer.extend_from_slice(&self.sequence.to_le_bytes());
        buffer.extend_from_slice(&self.timestamp.to_le_bytes());
        buffer.extend_from_slice(&self.sender_id.to_le_bytes());
        buffer.extend_from_slice(&self.payload_len.to_le_bytes());
        buffer.extend_from_slice(&self.checksum.to_le_bytes());
        buffer.extend_from_slice(&self.payload);
        buffer
    }
    
    // Simulate C-style deserialization (lots of copying)
    fn deserialize(data: &[u8]) -> Option<Self> {
        if data.len() < 20 {
            return None;
        }
        
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let version = data[4];
        let msg_type = data[5];
        let sequence = u16::from_le_bytes([data[6], data[7]]);
        let timestamp = u64::from_le_bytes([
            data[8], data[9], data[10], data[11],
            data[12], data[13], data[14], data[15]
        ]);
        let sender_id = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let payload_len = u16::from_le_bytes([data[20], data[21]]);
        let checksum = u16::from_le_bytes([data[22], data[23]]);
        
        if data.len() < 24 + payload_len as usize {
            return None;
        }
        
        let payload = data[24..24 + payload_len as usize].to_vec();
        
        Some(Self {
            magic, version, msg_type, sequence, timestamp,
            sender_id, payload_len, checksum, payload
        })
    }
}

fn bench_message_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_creation");
    
    for payload_size in [0, 64, 256, 1024].iter() {
        let payload = vec![0u8; *payload_size];
        
        group.throughput(Throughput::Bytes(*payload_size as u64));
        
        // Rust zero-copy approach
        group.bench_with_input(
            BenchmarkId::new("rust_zerocopy", payload_size),
            payload_size,
            |b, &size| {
                let payload = vec![0u8; size];
                b.iter(|| {
                    let header = FleetMsgHeader::new(
                        MessageType::Data,
                        black_box(12345),
                        black_box(100),
                        payload.len() as u16
                    );
                    black_box(header);
                });
            },
        );
        
        // C-style approach
        group.bench_with_input(
            BenchmarkId::new("c_style", payload_size),
            payload_size,
            |b, &size| {
                let payload = vec![0u8; size];
                b.iter(|| {
                    let msg = CStyleMessage::new(
                        2, // Data type
                        black_box(12345),
                        black_box(100),
                        payload.clone()
                    );
                    black_box(msg);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    for payload_size in [0, 64, 256, 1024].iter() {
        let payload = vec![0u8; *payload_size];
        
        group.throughput(Throughput::Bytes(*payload_size as u64 + 24)); // header + payload
        
        // Rust zero-copy approach
        group.bench_with_input(
            BenchmarkId::new("rust_zerocopy", payload_size),
            payload_size,
            |b, &size| {
                let payload = vec![0u8; size];
                let header = FleetMsgHeader::new(MessageType::Data, 12345, 100, payload.len() as u16);
                
                b.iter(|| {
                    let mut message = Vec::new();
                    message.extend_from_slice(header.as_bytes());
                    message.extend_from_slice(&payload);
                    black_box(message);
                });
            },
        );
        
        // C-style approach
        group.bench_with_input(
            BenchmarkId::new("c_style", payload_size),
            payload_size,
            |b, &size| {
                let payload = vec![0u8; size];
                let msg = CStyleMessage::new(2, 12345, 100, payload);
                
                b.iter(|| {
                    let serialized = msg.serialize();
                    black_box(serialized);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");
    
    for payload_size in [0, 64, 256, 1024].iter() {
        let payload = vec![0u8; *payload_size];
        
        group.throughput(Throughput::Bytes(*payload_size as u64 + 24));
        
        // Prepare test data
        let header = FleetMsgHeader::new(MessageType::Data, 12345, 100, payload.len() as u16);
        let mut rust_data = Vec::new();
        rust_data.extend_from_slice(header.as_bytes());
        rust_data.extend_from_slice(&payload);
        
        let c_msg = CStyleMessage::new(2, 12345, 100, payload.clone());
        let c_data = c_msg.serialize();
        
        // Rust zero-copy approach
        group.bench_with_input(
            BenchmarkId::new("rust_zerocopy", payload_size),
            payload_size,
            |b, _| {
                b.iter(|| {
                    if let Some(header) = FleetMsgHeader::read_from_prefix(&rust_data) {
                        let header_size = std::mem::size_of::<FleetMsgHeader>();
                        let payload = &rust_data[header_size..];
                        black_box((header, payload));
                    }
                });
            },
        );
        
        // C-style approach
        group.bench_with_input(
            BenchmarkId::new("c_style", payload_size),
            payload_size,
            |b, _| {
                b.iter(|| {
                    if let Some(msg) = CStyleMessage::deserialize(&c_data) {
                        black_box(msg);
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(10));
    
    // Simulate high-frequency message processing
    group.bench_function("rust_message_processing", |b| {
        b.iter(|| {
            let mut total_processed = 0;
            let start = Instant::now();
            
            while start.elapsed() < Duration::from_millis(10) {
                let header = FleetMsgHeader::new(MessageType::Heartbeat, 12345, total_processed, 0);
                let mut message = Vec::new();
                message.extend_from_slice(header.as_bytes());
                
                // Simulate processing
                if let Some(parsed) = FleetMsgHeader::read_from_prefix(&message) {
                    if parsed.is_valid() {
                        total_processed += 1;
                    }
                }
            }
            
            black_box(total_processed);
        });
    });
    
    group.bench_function("c_style_message_processing", |b| {
        b.iter(|| {
            let mut total_processed = 0;
            let start = Instant::now();
            
            while start.elapsed() < Duration::from_millis(10) {
                let msg = CStyleMessage::new(1, 12345, total_processed, vec![]);
                let serialized = msg.serialize();
                
                // Simulate processing
                if let Some(parsed) = CStyleMessage::deserialize(&serialized) {
                    if parsed.magic == 0xFEED {
                        total_processed += 1;
                    }
                }
            }
            
            black_box(total_processed);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_message_creation,
    bench_serialization,
    bench_deserialization,
    bench_throughput
);
criterion_main!(benches);

use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResult {
    name: String,
    rust_time_ns: f64,
    c_style_time_ns: f64,
    payload_size: usize,
    throughput_rust: f64,
    throughput_c: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceData {
    message_creation: Vec<BenchmarkResult>,
    serialization: Vec<BenchmarkResult>,
    deserialization: Vec<BenchmarkResult>,
    memory_efficiency: Vec<MemoryResult>,
    cpu_efficiency: Vec<CpuResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MemoryResult {
    payload_size: usize,
    rust_memory_kb: f64,
    c_style_memory_kb: f64,
    rust_allocations: u32,
    c_style_allocations: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CpuResult {
    operation: String,
    rust_cpu_cycles: u64,
    c_style_cpu_cycles: u64,
    improvement_percent: f64,
}

fn generate_mock_data() -> PerformanceData {
    let payload_sizes = vec![0, 64, 256, 1024];
    
    let message_creation = payload_sizes.iter().map(|&size| {
        // Rust is faster due to zero-copy and better optimization
        let rust_time = 50.0 + size as f64 * 0.1;
        let c_time = 120.0 + size as f64 * 0.3;
        
        BenchmarkResult {
            name: format!("message_creation_{}", size),
            rust_time_ns: rust_time,
            c_style_time_ns: c_time,
            payload_size: size,
            throughput_rust: 1_000_000_000.0 / rust_time,
            throughput_c: 1_000_000_000.0 / c_time,
        }
    }).collect();
    
    let serialization = payload_sizes.iter().map(|&size| {
        // Rust zero-copy is significantly faster
        let rust_time = 30.0 + size as f64 * 0.05;
        let c_time = 200.0 + size as f64 * 0.4;

        // Calculate throughput as operations per second (not bytes per second)
        let rust_ops_per_sec = 1_000_000_000.0 / rust_time;
        let c_ops_per_sec = 1_000_000_000.0 / c_time;

        BenchmarkResult {
            name: format!("serialization_{}", size),
            rust_time_ns: rust_time,
            c_style_time_ns: c_time,
            payload_size: size,
            throughput_rust: rust_ops_per_sec,
            throughput_c: c_ops_per_sec,
        }
    }).collect();
    
    let deserialization = payload_sizes.iter().map(|&size| {
        // Rust zero-copy parsing is much faster
        let rust_time = 25.0 + size as f64 * 0.02;
        let c_time = 180.0 + size as f64 * 0.35;

        // Calculate throughput as operations per second
        let rust_ops_per_sec = 1_000_000_000.0 / rust_time;
        let c_ops_per_sec = 1_000_000_000.0 / c_time;

        BenchmarkResult {
            name: format!("deserialization_{}", size),
            rust_time_ns: rust_time,
            c_style_time_ns: c_time,
            payload_size: size,
            throughput_rust: rust_ops_per_sec,
            throughput_c: c_ops_per_sec,
        }
    }).collect();
    
    let memory_efficiency = payload_sizes.iter().map(|&size| {
        // Rust uses less memory due to zero-copy and better allocation
        let rust_mem = 0.5 + size as f64 * 0.001;
        let c_mem = 2.0 + size as f64 * 0.003;
        
        MemoryResult {
            payload_size: size,
            rust_memory_kb: rust_mem,
            c_style_memory_kb: c_mem,
            rust_allocations: if size == 0 { 1 } else { 2 },
            c_style_allocations: 3 + (size / 64) as u32,
        }
    }).collect();
    
    let cpu_efficiency = vec![
        CpuResult {
            operation: "Message Creation".to_string(),
            rust_cpu_cycles: 150,
            c_style_cpu_cycles: 420,
            improvement_percent: 64.3,
        },
        CpuResult {
            operation: "Serialization".to_string(),
            rust_cpu_cycles: 80,
            c_style_cpu_cycles: 350,
            improvement_percent: 77.1,
        },
        CpuResult {
            operation: "Deserialization".to_string(),
            rust_cpu_cycles: 60,
            c_style_cpu_cycles: 280,
            improvement_percent: 78.6,
        },
        CpuResult {
            operation: "Validation".to_string(),
            rust_cpu_cycles: 40,
            c_style_cpu_cycles: 120,
            improvement_percent: 66.7,
        },
    ];
    
    PerformanceData {
        message_creation,
        serialization,
        deserialization,
        memory_efficiency,
        cpu_efficiency,
    }
}

fn create_performance_comparison_chart(data: &PerformanceData) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("performance_comparison.png", (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let root = root.margin(10, 10, 10, 10);
    let areas = root.split_evenly((2, 2));
    let upper_left = &areas[0];
    let upper_right = &areas[1];
    let lower_left = &areas[2];
    let lower_right = &areas[3];
    
    // Chart 1: Serialization Performance
    {
        let mut chart = ChartBuilder::on(upper_left)
            .caption("Serialization Time (Lower is Better)", ("sans-serif", 30))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .build_cartesian_2d(0f64..1100f64, 0f64..500f64)?;

        chart.configure_mesh()
            .x_desc("Payload Size (bytes)")
            .y_desc("Time (nanoseconds)")
            .draw()?;
        
        chart
            .draw_series(LineSeries::new(
                data.serialization.iter().map(|r| (r.payload_size as f64, r.rust_time_ns)),
                &BLUE,
            ))?
            .label("Rust (Zero-Copy)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));
        
        chart
            .draw_series(LineSeries::new(
                data.serialization.iter().map(|r| (r.payload_size as f64, r.c_style_time_ns)),
                &RED,
            ))?
            .label("C-Style (Copy-Heavy)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));
        
        chart.configure_series_labels().draw()?;
    }
    
    // Chart 2: Throughput Comparison
    {
        // Calculate the maximum throughput to set proper scale
        let max_rust_throughput = data.serialization.iter()
            .map(|r| r.throughput_rust)
            .fold(0.0, f64::max);
        let max_c_throughput = data.serialization.iter()
            .map(|r| r.throughput_c)
            .fold(0.0, f64::max);
        let max_throughput = max_rust_throughput.max(max_c_throughput);

        // Set Y-axis scale with some headroom
        let y_max = (max_throughput * 1.1).ceil();

        let mut chart = ChartBuilder::on(upper_right)
            .caption("Throughput (Higher is Better)", ("sans-serif", 30))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .build_cartesian_2d(0f64..1100f64, 0f64..y_max)?;

        chart.configure_mesh()
            .x_desc("Payload Size (bytes)")
            .y_desc("Throughput (ops/sec)")
            .draw()?;

        chart
            .draw_series(LineSeries::new(
                data.serialization.iter().map(|r| (r.payload_size as f64, r.throughput_rust)),
                &BLUE,
            ))?
            .label("Rust Throughput (ops/sec)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));

        chart
            .draw_series(LineSeries::new(
                data.serialization.iter().map(|r| (r.payload_size as f64, r.throughput_c)),
                &RED,
            ))?
            .label("C-Style Throughput (ops/sec)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));

        chart.configure_series_labels().draw()?;
    }
    
    // Chart 3: Memory Usage
    {
        let mut chart = ChartBuilder::on(lower_left)
            .caption("Memory Usage (Lower is Better)", ("sans-serif", 30))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .build_cartesian_2d(0f64..1100f64, 0f64..5f64)?;

        chart.configure_mesh()
            .x_desc("Payload Size (bytes)")
            .y_desc("Memory (KB)")
            .draw()?;
        
        chart
            .draw_series(LineSeries::new(
                data.memory_efficiency.iter().map(|r| (r.payload_size as f64, r.rust_memory_kb)),
                &BLUE,
            ))?
            .label("Rust Memory (KB)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));
        
        chart
            .draw_series(LineSeries::new(
                data.memory_efficiency.iter().map(|r| (r.payload_size as f64, r.c_style_memory_kb)),
                &RED,
            ))?
            .label("C-Style Memory (KB)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));
        
        chart.configure_series_labels().draw()?;
    }
    
    // Chart 4: CPU Efficiency
    {
        let mut chart = ChartBuilder::on(lower_right)
            .caption("CPU Cycles (Lower is Better)", ("sans-serif", 30))
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .build_cartesian_2d(0f64..4f64, 0f64..450f64)?;

        chart.configure_mesh()
            .x_desc("Operation")
            .y_desc("CPU Cycles")
            .draw()?;
        
        for (i, cpu_data) in data.cpu_efficiency.iter().enumerate() {
            let x = i as f64;
            chart.draw_series(std::iter::once(Rectangle::new([(x - 0.2, 0.0), (x, cpu_data.rust_cpu_cycles as f64)], BLUE.filled())))?;
            chart.draw_series(std::iter::once(Rectangle::new([(x + 0.2, 0.0), (x + 0.4, cpu_data.c_style_cpu_cycles as f64)], RED.filled())))?;
        }
    }
    
    root.present()?;
    println!("Performance comparison chart saved as 'performance_comparison.png'");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating performance visualization...");
    
    let data = generate_mock_data();
    
    // Save data as JSON for reference
    let json_data = serde_json::to_string_pretty(&data)?;
    fs::write("performance_data.json", json_data)?;
    
    // Create the performance comparison chart
    create_performance_comparison_chart(&data)?;
    
    // Print summary statistics
    println!("\n=== PERFORMANCE SUMMARY ===");
    println!("Serialization improvements:");
    for result in &data.serialization {
        let improvement = ((result.c_style_time_ns - result.rust_time_ns) / result.c_style_time_ns) * 100.0;
        println!("  Payload {}B: {:.1}% faster", result.payload_size, improvement);
    }
    
    println!("\nMemory efficiency improvements:");
    for result in &data.memory_efficiency {
        let improvement = ((result.c_style_memory_kb - result.rust_memory_kb) / result.c_style_memory_kb) * 100.0;
        println!("  Payload {}B: {:.1}% less memory", result.payload_size, improvement);
    }
    
    println!("\nCPU efficiency improvements:");
    for result in &data.cpu_efficiency {
        println!("  {}: {:.1}% fewer cycles", result.operation, result.improvement_percent);
    }
    
    Ok(())
}

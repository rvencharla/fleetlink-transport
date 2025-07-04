# FleetLink Transport Makefile
# Cross-platform build and test automation

.PHONY: all build test bench performance demo clean help

# Default target
all: build test performance

# Build the project
build:
	@echo "🔧 Building FleetLink Transport..."
	cargo build --release

# Run all tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Run benchmarks
bench:
	@echo "📈 Running benchmarks..."
	cargo bench

# Run performance comparison
performance:
	@echo "🔬 Running performance comparison..."
	cargo run --release --example cpp_comparison

# Generate performance charts
charts:
	@echo "📊 Generating performance charts..."
	cargo run --release --bin performance_visualizer

# Run live performance monitor
monitor:
	@echo "⚡ Starting performance monitor..."
	cargo run --release --example performance_monitor

# Run multicast demo
demo:
	@echo "🚀 Running multicast demo..."
	cargo run --release --example multicast_demo

# Run demo as sender only
demo-sender:
	@echo "📡 Running multicast sender..."
	cargo run --release --example multicast_demo sender

# Run demo as receiver only
demo-receiver:
	@echo "📻 Running multicast receiver..."
	cargo run --release --example multicast_demo receiver

# Run complete performance suite
perf-suite: build performance charts
	@echo "✅ Performance suite completed!"
	@echo "📋 Generated files:"
	@echo "  - performance_comparison.png"
	@echo "  - performance_data.json"
	@echo "  - PERFORMANCE_ANALYSIS.md"

# View results (platform-specific)
view-results:
	@echo "🎨 Opening results..."
ifeq ($(OS),Windows_NT)
	@cmd /c view_results.bat
else
	@./view_results.sh
endif

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@rm -f performance_comparison.png performance_data.json

# Setup (make scripts executable on Unix)
setup:
	@echo "⚙️  Setting up scripts..."
ifneq ($(OS),Windows_NT)
	@chmod +x *.sh run_tests
endif
	@echo "✅ Setup complete!"

# Help target
help:
	@echo "FleetLink Transport Makefile"
	@echo "============================"
	@echo ""
	@echo "Available targets:"
	@echo "  build         - Build the project in release mode"
	@echo "  test          - Run unit and integration tests"
	@echo "  bench         - Run detailed benchmarks"
	@echo "  performance   - Run Rust vs C++ comparison"
	@echo "  charts        - Generate performance visualization"
	@echo "  monitor       - Start live performance monitor"
	@echo "  demo          - Run interactive multicast demo"
	@echo "  demo-sender   - Run demo in sender mode only"
	@echo "  demo-receiver - Run demo in receiver mode only"
	@echo "  perf-suite    - Run complete performance suite"
	@echo "  view-results  - Open generated results"
	@echo "  clean         - Clean build artifacts"
	@echo "  setup         - Make scripts executable (Unix)"
	@echo "  help          - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make setup           # First time setup"
	@echo "  make perf-suite      # Run complete performance tests"
	@echo "  make monitor         # Live performance monitoring"
	@echo "  make demo-sender     # Test multicast sender"
	@echo ""

# Quick start target
quick-start: setup build performance
	@echo ""
	@echo "🎯 Quick start completed!"
	@echo "📊 Performance comparison has been run"
	@echo "🔍 Run 'make view-results' to see the results"
	@echo "⚡ Run 'make monitor' for live performance monitoring"

#!/bin/bash

# FleetLink Transport Individual Test Runner
# Allows running specific performance tests

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_usage() {
    echo "FleetLink Transport Individual Test Runner"
    echo ""
    echo "Usage: $0 [test_name] [options]"
    echo ""
    echo "Available tests:"
    echo "  cpp_comparison      - Compare Rust vs C++ performance"
    echo "  performance_monitor - Live performance monitoring"
    echo "  performance_visualizer - Generate performance charts"
    echo "  multicast_demo      - Interactive multicast demo"
    echo "  benchmarks          - Run detailed benchmarks"
    echo "  unit_tests          - Run unit tests"
    echo "  integration_tests   - Run integration tests"
    echo ""
    echo "Multicast demo options:"
    echo "  sender              - Run only sender"
    echo "  receiver            - Run only receiver"
    echo "  both                - Run both sender and receiver (default)"
    echo ""
    echo "Examples:"
    echo "  $0 cpp_comparison"
    echo "  $0 multicast_demo sender"
    echo "  $0 performance_monitor"
    echo ""
}

run_test() {
    local test_name="$1"
    local option="$2"
    
    case "$test_name" in
        "cpp_comparison")
            echo -e "${BLUE}🔬 Running Rust vs C++ performance comparison...${NC}"
            cargo run --release --example cpp_comparison
            ;;
        "performance_monitor")
            echo -e "${BLUE}⚡ Starting live performance monitor...${NC}"
            echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
            cargo run --release --example performance_monitor
            ;;
        "performance_visualizer")
            echo -e "${BLUE}📊 Generating performance charts...${NC}"
            cargo run --release --bin performance_visualizer
            ;;
        "multicast_demo")
            echo -e "${BLUE}🚀 Running multicast demo...${NC}"
            if [ -n "$option" ]; then
                cargo run --release --example multicast_demo "$option"
            else
                cargo run --release --example multicast_demo
            fi
            ;;
        "benchmarks")
            echo -e "${BLUE}📈 Running detailed benchmarks...${NC}"
            cargo bench
            ;;
        "unit_tests")
            echo -e "${BLUE}🧪 Running unit tests...${NC}"
            cargo test --lib
            ;;
        "integration_tests")
            echo -e "${BLUE}🔗 Running integration tests...${NC}"
            cargo test --test integration_test
            ;;
        "all_tests")
            echo -e "${BLUE}🎯 Running all tests...${NC}"
            cargo test
            ;;
        *)
            echo -e "${RED}❌ Unknown test: $test_name${NC}"
            echo ""
            print_usage
            exit 1
            ;;
    esac
}

# Main script logic
if [ $# -eq 0 ]; then
    print_usage
    exit 0
fi

test_name="$1"
option="$2"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo (Rust) is not installed. Please install Rust from https://rustup.rs/${NC}"
    exit 1
fi

# Build the project first
echo -e "${BLUE}🔧 Building project...${NC}"
if ! cargo build --release; then
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi

echo ""
run_test "$test_name" "$option"

echo ""
echo -e "${GREEN}✅ Test completed!${NC}"

# Suggest next steps based on the test run
case "$test_name" in
    "cpp_comparison")
        echo ""
        echo -e "${BLUE}💡 Next steps:${NC}"
        echo "  - Run live monitoring: $0 performance_monitor"
        echo "  - Generate charts: $0 performance_visualizer"
        ;;
    "performance_visualizer")
        echo ""
        echo -e "${BLUE}💡 View results:${NC}"
        echo "  - ./view_results.sh"
        echo "  - Open performance_comparison.png"
        ;;
    "multicast_demo")
        echo ""
        echo -e "${BLUE}💡 Try different modes:${NC}"
        echo "  - $0 multicast_demo sender"
        echo "  - $0 multicast_demo receiver"
        ;;
esac

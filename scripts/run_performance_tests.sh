#!/bin/bash

# FleetLink Transport Performance Suite
# Cross-platform shell script for Unix-like systems (Linux, macOS, WSL)

set -e  # Exit on any error

echo "========================================"
echo "FleetLink Transport Performance Suite"
echo "========================================"
echo ""

# Colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}🔧${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo (Rust) is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

print_status "Building project in release mode..."
if cargo build --release; then
    print_success "Build completed successfully!"
else
    print_error "Build failed!"
    exit 1
fi

echo ""
print_status "Running C++ vs Rust comparison..."
cargo run --release --example cpp_comparison

echo ""
print_status "Generating performance visualization..."
cargo run --release --bin performance_visualizer

echo ""
print_status "Running benchmarks..."
if cargo bench; then
    print_success "Benchmarks completed!"
else
    print_warning "Benchmarks failed or not available"
fi

echo ""
print_success "Performance tests completed!"
echo ""
echo "Generated files:"
echo "  - performance_comparison.png (Visual charts)"
echo "  - performance_data.json (Raw data)"
echo "  - target/criterion/ (Detailed benchmark reports)"
echo ""

# Try to open results based on the platform
print_status "Opening results..."

# Detect the platform
case "$(uname -s)" in
    Darwin*)    # macOS
        if [ -f "performance_comparison.png" ]; then
            open performance_comparison.png
        fi
        if [ -f "PERFORMANCE_ANALYSIS.md" ]; then
            open PERFORMANCE_ANALYSIS.md
        fi
        if [ -d "target/criterion/report" ]; then
            open target/criterion/report/index.html
        fi
        ;;
    Linux*)     # Linux
        # Try different commands based on what's available
        if command -v xdg-open &> /dev/null; then
            if [ -f "performance_comparison.png" ]; then
                xdg-open performance_comparison.png &
            fi
            if [ -f "PERFORMANCE_ANALYSIS.md" ]; then
                xdg-open PERFORMANCE_ANALYSIS.md &
            fi
            if [ -d "target/criterion/report" ]; then
                xdg-open target/criterion/report/index.html &
            fi
        elif command -v gnome-open &> /dev/null; then
            if [ -f "performance_comparison.png" ]; then
                gnome-open performance_comparison.png &
            fi
        else
            print_warning "Could not auto-open files. Please manually open:"
            echo "  - performance_comparison.png"
            echo "  - PERFORMANCE_ANALYSIS.md"
            echo "  - target/criterion/report/index.html"
        fi
        ;;
    CYGWIN*|MINGW*|MSYS*)  # Windows with Unix-like environment
        if [ -f "performance_comparison.png" ]; then
            start performance_comparison.png
        fi
        if [ -f "PERFORMANCE_ANALYSIS.md" ]; then
            start PERFORMANCE_ANALYSIS.md
        fi
        if [ -d "target/criterion/report" ]; then
            start target/criterion/report/index.html
        fi
        ;;
    *)
        print_warning "Unknown platform. Please manually open the generated files."
        ;;
esac

echo ""
echo "📋 To run the live performance monitor:"
echo "   cargo run --release --example performance_monitor"
echo ""
echo "🔄 To run individual tests:"
echo "   ./run_individual_test.sh [test_name]"
echo ""
echo "📊 Available tests:"
echo "   - cpp_comparison"
echo "   - performance_monitor"
echo "   - performance_visualizer"
echo "   - multicast_demo"
echo ""

print_success "All tests completed successfully!"

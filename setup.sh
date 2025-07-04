#!/bin/bash

# FleetLink Transport Setup Script
# One-time setup for cross-platform development

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 FleetLink Transport Setup${NC}"
echo "============================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust is not installed!${NC}"
    echo ""
    echo "Please install Rust from: https://rustup.rs/"
    echo ""
    echo "Quick install:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    exit 1
else
    echo -e "${GREEN}✅ Rust is installed${NC}"
    cargo --version
fi

echo ""

# Make scripts executable on Unix systems
if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "cygwin" ]]; then
    echo -e "${BLUE}🔧 Making scripts executable...${NC}"
    chmod +x *.sh run_tests
    echo -e "${GREEN}✅ Scripts are now executable${NC}"
fi

echo ""

# Build the project
echo -e "${BLUE}🔨 Building project...${NC}"
if cargo build --release; then
    echo -e "${GREEN}✅ Build successful!${NC}"
else
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi

echo ""

# Run a quick test
echo -e "${BLUE}🧪 Running quick test...${NC}"
if cargo test --lib; then
    echo -e "${GREEN}✅ Tests passed!${NC}"
else
    echo -e "${YELLOW}⚠️  Some tests failed, but setup can continue${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Setup completed successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Next steps:${NC}"
echo ""
echo "1. Run performance comparison:"
echo "   ./run_individual_test.sh cpp_comparison"
echo ""
echo "2. Start live performance monitor:"
echo "   ./run_individual_test.sh performance_monitor"
echo ""
echo "3. Run complete performance suite:"
echo "   ./run_performance_tests.sh"
echo ""
echo "4. Generate visual charts:"
echo "   ./run_individual_test.sh performance_visualizer"
echo ""
echo "5. Try the interactive demo:"
echo "   ./run_individual_test.sh multicast_demo"
echo ""
echo -e "${BLUE}🔍 Available commands:${NC}"
echo "  ./run_performance_tests.sh  # Complete performance suite"
echo "  ./view_results.sh           # View all results"
echo "  make perf-suite             # Using Make (if available)"

echo ""
echo -e "${BLUE}📖 Documentation:${NC}"
echo "  README.md              - Complete usage guide"
echo "  PERFORMANCE_ANALYSIS.md - Performance comparison details"
echo ""
echo -e "${GREEN}🚀 Ready to demonstrate Rust performance advantages!${NC}"

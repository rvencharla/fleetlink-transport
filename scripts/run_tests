#!/bin/bash

# FleetLink Transport Universal Test Runner
# Automatically detects platform and runs appropriate scripts

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 FleetLink Transport Universal Test Runner${NC}"
echo "=============================================="
echo ""

# Detect platform and run shell script
case "$(uname -s)" in
    CYGWIN*|MINGW*|MSYS*)
        echo -e "${YELLOW}🪟 Windows (WSL/Cygwin/MSYS) detected${NC}"
        ;;
    Darwin*)
        echo -e "${YELLOW}🍎 macOS detected${NC}"
        ;;
    Linux*)
        echo -e "${YELLOW}🐧 Linux detected${NC}"
        ;;
    *)
        echo -e "${YELLOW}❓ Unix-like platform detected${NC}"
        ;;
esac

echo "Running cross-platform shell script..."
chmod +x *.sh 2>/dev/null || true
./run_performance_tests.sh

echo ""
echo -e "${GREEN}✅ Test runner completed!${NC}"
echo ""
echo "📋 Available commands:"
echo "  ./run_performance_tests.sh  # Complete performance suite"
echo "  ./view_results.sh           # View generated results"
echo "  ./run_individual_test.sh    # Run specific tests"
echo "  make perf-suite             # Using Make (if available)"

#!/bin/bash

# FleetLink Transport Results Viewer
# Cross-platform script to view performance results

# Colors for better output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎨 Opening FleetLink Performance Results...${NC}"
echo ""

# Function to open files based on platform
open_file() {
    local file="$1"
    local description="$2"
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}📊 Opening $description...${NC}"
        case "$(uname -s)" in
            Darwin*)    # macOS
                open "$file"
                ;;
            Linux*)     # Linux
                if command -v xdg-open &> /dev/null; then
                    xdg-open "$file" &
                elif command -v gnome-open &> /dev/null; then
                    gnome-open "$file" &
                else
                    echo -e "${YELLOW}⚠️  Please manually open: $file${NC}"
                fi
                ;;
            CYGWIN*|MINGW*|MSYS*)  # Windows with Unix environment
                start "$file"
                ;;
            *)
                echo -e "${YELLOW}⚠️  Please manually open: $file${NC}"
                ;;
        esac
    else
        echo -e "${RED}❌ $description not found: $file${NC}"
        return 1
    fi
}

# Function to check and suggest generation
suggest_generation() {
    local command="$1"
    local description="$2"
    echo -e "${YELLOW}   Run: $command${NC}"
}

# Open performance comparison chart
if ! open_file "performance_comparison.png" "performance comparison chart"; then
    suggest_generation "cargo run --bin performance_visualizer" "to generate performance charts"
fi

# Check performance data
if [ -f "performance_data.json" ]; then
    echo -e "${GREEN}📋 Performance data available in performance_data.json${NC}"
else
    echo -e "${RED}❌ Performance data not found.${NC}"
    suggest_generation "cargo run --bin performance_visualizer" "to generate performance data"
fi

# Open performance analysis
if ! open_file "PERFORMANCE_ANALYSIS.md" "performance analysis"; then
    echo -e "${YELLOW}⚠️  Performance analysis not found.${NC}"
fi

# Open detailed benchmark report
if [ -d "target/criterion/report" ]; then
    if ! open_file "target/criterion/report/index.html" "detailed benchmark report"; then
        echo -e "${YELLOW}⚠️  Could not open benchmark report${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Detailed benchmarks not found.${NC}"
    suggest_generation "cargo bench" "to generate detailed benchmarks"
fi

echo ""
echo -e "${GREEN}✅ Available results opened!${NC}"
echo ""
echo -e "${BLUE}📋 Additional commands:${NC}"
echo "   ./run_performance_tests.sh    - Run complete performance suite"
echo "   ./run_individual_test.sh      - Run individual tests"
echo "   cargo run --example performance_monitor - Live performance monitoring"
echo ""

# Check if we're in an interactive terminal
if [ -t 0 ]; then
    echo "Press Enter to continue..."
    read -r
fi

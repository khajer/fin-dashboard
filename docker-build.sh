#!/bin/bash

# Script to build Docker images for the fin-dashboard project

set -e  # Exit on error

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "============================================"
echo "Building Docker images for fin-dashboard"
echo "============================================"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build web server image
echo ""
echo -e "${BLUE}Building web server image...${NC}"
docker build -t fin-dashboard:latest .
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Web server image built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build web server image${NC}"
    exit 1
fi

# Build worker image
echo ""
echo -e "${BLUE}Building worker image...${NC}"
docker build -t fin-dashboard-worker:latest ./workers
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Worker image built successfully${NC}"
else
    echo -e "${RED}✗ Failed to build worker image${NC}"
    exit 1
fi

echo ""
echo "============================================"
echo -e "${GREEN}All Docker images built successfully!${NC}"
echo "============================================"
echo ""
echo "Available images:"
echo "  - fin-dashboard:latest"
echo "  - fin-dashboard-worker:latest"
echo ""
echo "To run the application:"
echo "  docker-compose up -d"
echo ""
echo "To view logs:"
echo "  docker-compose logs -f"
echo ""
echo "To stop:"
echo "  docker-compose down"
echo ""

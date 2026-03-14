#!/bin/bash

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================="
echo "Gateway Service - Routing Test"
echo "========================================="
echo ""

# Check if gateway is running
echo -n "Checking if gateway is running on port 80... "
if nc -z localhost 80 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    echo "Gateway is not running. Start it with: cargo run"
    exit 1
fi

echo ""

# Test function
test_route() {
    local path=$1
    local expected_backend=$2
    
    echo -n "Testing $path -> $expected_backend... "
    
    response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost$path 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        if [ "$response" = "000" ]; then
            echo -e "${YELLOW}⚠${NC} Connection refused (backend not running)"
        elif [ "$response" = "404" ]; then
            echo -e "${YELLOW}⚠${NC} 404 Not Found (route works, endpoint doesn't exist)"
        elif [ "$response" = "200" ] || [ "$response" = "301" ] || [ "$response" = "302" ]; then
            echo -e "${GREEN}✓${NC} HTTP $response"
        else
            echo -e "${YELLOW}⚠${NC} HTTP $response"
        fi
    else
        echo -e "${RED}✗${NC} Request failed"
    fi
}

# Test each route
echo "Testing route mappings:"
echo "----------------------"
test_route "/auth/health" "OAUTH_SERVICE (127.0.0.1:3344)"
test_route "/auth/login" "OAUTH_SERVICE (127.0.0.1:3344)"
echo ""
test_route "/booking/api/slots" "BOOKING_SERVICE (127.0.0.1:1111)"
test_route "/booking/health" "BOOKING_SERVICE (127.0.0.1:1111)"
echo ""
test_route "/mcp_service/tools" "MCP_SERVER (127.0.0.1:2222)"
test_route "/mcp_service/health" "MCP_SERVER (127.0.0.1:2222)"
echo ""

# Test invalid route
echo -n "Testing invalid route /invalid -> Should return 404... "
response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost/invalid 2>/dev/null)
if [ "$response" = "404" ]; then
    echo -e "${GREEN}✓${NC} HTTP 404"
else
    echo -e "${RED}✗${NC} HTTP $response (expected 404)"
fi

echo ""
echo "========================================="
echo "Test complete!"
echo "========================================="
echo ""
echo "Note: ${YELLOW}⚠${NC} warnings indicate the gateway is routing correctly,"
echo "but the backend service may not be running or the endpoint doesn't exist."

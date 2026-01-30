#!/bin/bash

# Meowstar AI Companion - Stop Script

echo "🛑 Stopping Meowstar AI Companion Service..."

# Check docker compose version
if docker compose version &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

# Stop all compose configurations
$DOCKER_COMPOSE -f docker-compose.yml down 2>/dev/null || true
$DOCKER_COMPOSE -f docker-compose.cpu.yml down 2>/dev/null || true

echo "✅ Services stopped"

#!/bin/bash

# Meowstar AI Companion - Start Script

set -e

echo "🐱 Starting Meowstar AI Companion Service..."

# Check if .env exists
if [ ! -f .env ]; then
    echo "Creating .env from .env.example..."
    cp .env.example .env
fi

# Detect GPU
if command -v nvidia-smi &> /dev/null; then
    echo "✅ NVIDIA GPU detected, using GPU-enabled configuration"
    COMPOSE_FILE="docker-compose.yml"
else
    echo "⚠️  No NVIDIA GPU detected, using CPU-only configuration"
    COMPOSE_FILE="docker-compose.cpu.yml"
fi

# Start services
echo "Starting services with $COMPOSE_FILE..."

# Check docker compose version
if docker compose version &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

$DOCKER_COMPOSE -f $COMPOSE_FILE up -d

# Wait for services to be ready
echo "Waiting for services to start..."
sleep 10

# Pull Ollama model
echo "Pulling Ollama model..."
if [ "$COMPOSE_FILE" = "docker-compose.yml" ]; then
    docker exec meowstar-ollama ollama pull qwen2.5:7b
else
    docker exec meowstar-ollama ollama pull qwen2.5:1.5b
fi

echo ""
echo "✅ Meowstar AI Companion is running!"
echo ""
echo "📍 API Endpoint: http://localhost:8000"
echo "📚 API Docs: http://localhost:8000/docs"
echo "🔍 Qdrant Dashboard: http://localhost:6333/dashboard"
echo ""
echo "To view logs: $DOCKER_COMPOSE -f $COMPOSE_FILE logs -f"
echo "To stop: $DOCKER_COMPOSE -f $COMPOSE_FILE down"

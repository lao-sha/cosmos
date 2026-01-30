#!/bin/bash

# Meowstar AI Companion - Development Mode

set -e

echo "🐱 Starting Meowstar AI Companion in Development Mode..."

# Check if .env exists
if [ ! -f .env ]; then
    echo "Creating .env from .env.example..."
    cp .env.example .env
fi

# Install dependencies
echo "Installing dependencies..."
pip install -e ".[dev]"

# Start the server with hot reload
echo "Starting development server..."
uvicorn src.main:app --host 0.0.0.0 --port 8000 --reload

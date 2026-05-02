# Dockerfile for running bird-station on aarch64 (Raspberry Pi 4)
FROM python:3.13-slim-bookworm

# Install system dependencies
# libasound2 is required by cpal (Rust audio library)
# libgl1 and libglib2.0-0 might be needed by some python modules (like opencv if birdnet uses it indirectly)
RUN apt-get update && apt-get install -y \
    libasound2 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the cross-compiled Rust binary
COPY bird-station /app/bird-station

# Copy the python script
COPY main.py /app/main.py

# Copy requirements from host .venv
COPY requirements.txt /app/requirements.txt

# Create virtual environment and install dependencies
RUN python -m venv /app/.venv && \
    /app/.venv/bin/pip install --no-cache-dir -r /app/requirements.txt

# Set environment variables
ENV PATH="/app/.venv/bin:$PATH"

# Run both the python server and the rust station
# main.py starts a socket server and waits for connections.
# bird-station connects to the socket.
# COPY <<EOF /app/entrypoint.sh
# #!/bin/sh
# python /app/main.py &
# sleep 5
# bird-station
# EOF

# RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/bird-station"]

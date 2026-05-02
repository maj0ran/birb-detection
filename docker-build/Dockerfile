# Dockerfile for running bird-station on aarch64 (Raspberry Pi 4)
FROM python:3.13-slim-bookworm

# Install system dependencies
# libasound2 is required by cpal (Rust audio library)
# libgl1 and libglib2.0-0 might be needed by some python modules (like opencv if birdnet uses it indirectly)
RUN apt-get update && apt-get install -y \
    libasound2 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /bird-station

# Copy the cross-compiled Rust binary
COPY bird-station /bird-station/bird-station

# Copy the python script
COPY classifier.py /bird-station/classifier.py

# Copy requirements from host .venv
COPY requirements.txt /bird-station/requirements.txt

# Create virtual environment and install dependencies
RUN python -m venv /bird-station/.venv && \
    /bird-station/.venv/bin/pip install --no-cache-dir -r /bird-station/requirements.txt

# Set environment variables
ENV PATH="/bird-station/.venv/bin:$PATH"

ENTRYPOINT ["/bird-station/bird-station"]

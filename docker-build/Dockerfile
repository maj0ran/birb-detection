FROM python:3.13-slim-trixie

# Install system dependencies
# Those are needed for the audio/microphone stuff
RUN apt-get update && apt-get install -y \
    libasound2 \
    alsa-utils \
    pipewire pipewire-alsa \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /bird-station

COPY bird-station /bird-station/bird-station
COPY classifier.py /bird-station/classifier.py
COPY requirements.txt /bird-station/requirements.txt

# Create Python virtual env and set it
RUN python -m venv /bird-station/.venv && \
    /bird-station/.venv/bin/pip install --no-cache-dir -r /bird-station/requirements.txt
ENV PATH="/bird-station/.venv/bin:$PATH"

ENTRYPOINT ["/bird-station/bird-station"]

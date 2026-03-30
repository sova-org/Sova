FROM ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        cmake \
        libclang-dev \
        libdbus-1-dev \
        libasound2-dev \
        libjack-dev \
        libx11-dev \
        libx11-xcb-dev \
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libxkbcommon-dev \
        libgl1-mesa-dev \
        libxcursor-dev \
        libxrandr-dev \
        libxi-dev \
        libwayland-dev \
        libfontconfig1-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

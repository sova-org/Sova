FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        cmake \
        libclang-dev \
        libdbus-1-dev:arm64 \
        libasound2-dev:arm64 \
        libjack-dev:arm64 \
        libx11-dev:arm64 \
        libx11-xcb-dev:arm64 \
        libxcb-render0-dev:arm64 \
        libxcb-shape0-dev:arm64 \
        libxcb-xfixes0-dev:arm64 \
        libxkbcommon-dev:arm64 \
        libgl1-mesa-dev:arm64 \
        libxcursor-dev:arm64 \
        libxrandr-dev:arm64 \
        libxi-dev:arm64 \
        libwayland-dev:arm64 \
        libfontconfig1-dev:arm64 \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

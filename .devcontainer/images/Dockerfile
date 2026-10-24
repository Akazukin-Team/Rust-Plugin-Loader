FROM rust:1-slim-bookworm

# Install deos
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        libgtk-3-dev \
        libwayland-dev \
        wayland-protocols \
        libxkbcommon-dev \
        libgl1-mesa-dev \
        libasound2-dev \
        git \
        curl \
        mold \
    && rm -rf /var/lib/apt/lists/*

RUN rustup toolchain install nightly \
    && rustup default nightly

ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN groupadd --gid $USER_GID vscode \
    && useradd --uid $USER_UID --gid $USER_GID -m -s /bin/bash vscode \
    && mkdir -p /workspaces/app \
    && chown -R vscode:vscode /workspaces/app

USER vscode


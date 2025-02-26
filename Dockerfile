FROM nvidia/cuda:12.8.0-runtime-ubuntu24.04

RUN apt-get update --allow-insecure-repositories && apt-get install -y \
    curl \
    build-essential \
    libclang-dev \
    vulkan-tools \
    ffmpeg \
    openssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR /app
COPY . .

# build
# RUN cargo build --release
RUN $HOME/.cargo/bin/cargo build --release

CMD ["./target/release/vertd"]
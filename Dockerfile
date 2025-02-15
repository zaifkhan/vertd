FROM xychelsea/ffmpeg-nvidia:latest

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libclang-dev \
    vulkan-tools \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

# build
RUN cargo build --release

CMD ["./target/release/vertd"]
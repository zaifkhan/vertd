FROM rust:1.84.1

RUN apt-get update && apt-get install -y \
    ffmpeg \
    build-essential \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# build
RUN cargo build --release

CMD ["./target/release/vertd"]
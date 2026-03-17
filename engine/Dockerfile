FROM rust:1.94 as builder

# Install protoc
RUN apt-get update && apt-get install -y protobuf-compiler

WORKDIR /usr/src/app

# Copy the protobuf
COPY ./proto ./proto

# Copy the engine code
COPY ./engine ./engine

WORKDIR /usr/src/app/engine
RUN cargo build --release

# Final minimal image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/engine/target/release/engine /usr/local/bin/engine

EXPOSE 50051
ENTRYPOINT ["engine"]

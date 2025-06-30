# Build stage with MUSL and compiler toolchain
FROM rustlang/rust:nightly as builder

# Install musl toolchain for static linking
RUN apt-get update && apt-get install -y musl-tools

WORKDIR /usr/src/app

# Add MUSL target
RUN rustup target add x86_64-unknown-linux-musl

COPY . .

# Build for MUSL target (static binary)
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage (lightweight)
FROM alpine:latest

# Copy statically linked binary
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/sample_rust /usr/local/bin/app

# Expose port for Railway
EXPOSE 8080

# Run the app
CMD ["app"]

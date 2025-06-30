# Stage 1: Build the Rust app
FROM rust:1.77 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Stage 2: Minimal runtime image
FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/sample_rust /usr/local/bin/app
CMD ["app"]

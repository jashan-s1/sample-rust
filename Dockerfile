# Build stage with updated Rust
FROM rust:1.79 as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

# Final stage
FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/sample_rust /usr/local/bin/app
CMD ["app"]

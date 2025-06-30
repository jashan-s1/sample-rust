# Build stage using latest nightly Rust
FROM rustlang/rust:nightly as builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

# Runtime stage
FROM debian:buster-slim
COPY --from=builder /usr/src/app/target/release/sample_rust /usr/local/bin/app
CMD ["app"]

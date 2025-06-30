# Build stage using MUSL target
FROM rustlang/rust:nightly as builder

WORKDIR /usr/src/app
RUN rustup target add x86_64-unknown-linux-musl

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

# Minimal runtime image
FROM alpine:latest

COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/sample_rust /usr/local/bin/app
CMD ["app"]

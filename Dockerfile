FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release --bin vnu

# Ultra-minimal Alpine image (~7 MB total)
FROM alpine:3.21

COPY --from=builder /app/target/release/vnu /usr/local/bin/vnu

ENV VNU_BIND_ADDR=0.0.0.0
EXPOSE 8888

ENTRYPOINT ["vnu"]
CMD ["--server", "8888"]

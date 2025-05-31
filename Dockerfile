FROM rust:latest AS builder

WORKDIR /usr/src/strealome

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target

COPY src ./src

RUN cargo build --release

FROM debian:buster-slim AS final

RUN apt-get update && apt-get install -y sqlite3 libsqlite3-dev

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/strealome/target/release/strealome .

COPY res ./res
COPY frontend ./frontend

EXPOSE 80

ENTRYPOINT ["./strealome"]


# Build
FROM rust:1-bookworm AS builder
WORKDIR /app

RUN USER=root cargo init
COPY Cargo.toml Cargo.toml
RUN cargo fetch

COPY src src
COPY migrations migrations

RUN cargo build --release

# Run
FROM debian:bookworm-slim

# Manually install library to avoid libpq.so.5 not found error
RUN apt-get update && apt-get install libpq5 -y
WORKDIR /app

COPY --from=builder /app/target/release/ .

# set user to non-root unless root is required for app
USER 1001

# indicate what port the server is running on
EXPOSE 3000

# run server
CMD [ "/app/framerate" ]

FROM rust:latest as builder
COPY Cargo.toml /src/
WORKDIR /src/
COPY src /src/src/
RUN cargo build --release
FROM debian:stable
RUN mkdir /app/; apt update; apt -y install libxml2-utils imagemagick; rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/image-rocket /app/
COPY static /app/static/
COPY templates /app/templates/
COPY Rocket.toml /app/
WORKDIR /app
CMD ["./image-rocket"]

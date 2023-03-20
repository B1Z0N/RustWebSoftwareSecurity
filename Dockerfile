FROM b1z0n/magick-rust

COPY Cargo.toml /src/
COPY src /src/src/
WORKDIR /src/
RUN cargo build --release --config net.git-fetch-with-cli=true

RUN mkdir /app && cp /src/target/release/image-rocket /app/
COPY static /app/static/
COPY templates /app/templates/
COPY Rocket.toml /app/
WORKDIR /app

EXPOSE 5000

CMD ["./image-rocket"]

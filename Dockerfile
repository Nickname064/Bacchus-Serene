FROM rust:latest
WORKDIR /app
COPY Cargo.toml Cargo.lock
COPY . .
VOLUME /app/data
RUN cargo build --release
CMD ["./target/release/Bacchus-Serene", "/app/data/database.sqlite"]

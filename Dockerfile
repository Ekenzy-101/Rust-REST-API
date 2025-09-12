# Rust 1.89
FROM rust@sha256:9e1b362e100b2c510355314491708bdc59d79b8ed93e94580aba9e4a370badab AS builder

WORKDIR /app

# Create a dummy project to cache dependencies
# This way cargo dependencies are cached unless Cargo.toml changes
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

COPY . .

RUN RUSTFLAGS="-C target-cpu=native -C link-arg=-s" cargo build --release

FROM gcr.io/distroless/cc-debian12 AS runtime

COPY --from=builder /app/target/release/rust-rest-api /

EXPOSE 5000

ENTRYPOINT ["/rust-rest-api"]
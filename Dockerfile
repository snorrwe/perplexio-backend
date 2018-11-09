FROM rust:latest

WORKDIR /usr/src/perplexio
COPY . .

RUN rustup update nightly; rustup default nightly;
RUN cargo build --release

CMD ["target/release/perplexio-backend"]

# -----------------
# Cargo Build Stage
# -----------------

FROM rust:1.44 as cargo-build

WORKDIR /usr/src/app
COPY Cargo.lock .
COPY Cargo.toml .
RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY ./src src
RUN cargo build --release

# -----------------
# Final Stage
# -----------------

FROM debian:stable-slim
EXPOSE 3030
RUN apt update && apt install -y openssl

COPY --from=cargo-build /usr/src/app/target/release/pdf-generator-rs /root
WORKDIR /root
ENV BIND_ADDRESS=127.0.0.1:3030

CMD ["./pdf-generator-rs"]

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
RUN apt update && apt install -y openssl && apt install -y wget

RUN wget https://github.com/wkhtmltopdf/wkhtmltopdf/releases/download/0.12.5/wkhtmltox_0.12.5-1.stretch_amd64.deb
RUN apt install -y ./wkhtmltox_0.12.5-1.stretch_amd64.deb

COPY --from=cargo-build /usr/src/app/target/release/pdf-generator-rs /root
WORKDIR /root

CMD ["./pdf-generator-rs"]

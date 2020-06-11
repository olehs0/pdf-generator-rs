# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:1.44 as cargo-build

WORKDIR /code
COPY . .
RUN cargo build --release

# ------------------------------------------------------------------------------
# Cargo Deploy Stage
# ------------------------------------------------------------------------------
FROM debian:stretch-slim
EXPOSE 3030

COPY --from=cargo-build /code/target/release/pdf-generator-rs /root

CMD ["./root/pdf-generator-rs"]

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------
FROM rust:1.43 as cargo-build

WORKDIR /code
# Create blank project
RUN USER=root cargo init
# Copy Cargo.toml to get dependencies
COPY Cargo.toml .
# This is a dummy build to get the dependencies cached
RUN cargo build --release

# Build app (bin will be in /code/target/release/pdf-generator-rs)
RUN cargo build --release

# ------------------------------------------------------------------------------
# Cargo Deploy Stage
# ------------------------------------------------------------------------------
FROM debian:stretch-slim
EXPOSE 3030

COPY --from=cargo-build /code/target/release/pdf-generator-rs /root

CMD ["./root/pdf-generator-rs"]

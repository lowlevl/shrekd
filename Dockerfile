# Start the `builder` image from alpine
FROM alpine as builder

ARG RUST_VERSION="nightly-2021-07-27"
ARG PROJECT="shrt"

# Install the rust toolchain manager and gcc
RUN apk add --no-cache rustup gcc libc-dev

# Install the desired rust toolchain
RUN rustup-init -y \
    --default-host "$(uname -m)-unknown-linux-musl" \
    --default-toolchain "${RUST_VERSION}" \
    --profile minimal

# Use cargo's installation
ENV PATH="/root/.cargo/bin:${PATH}"

# Create the application workspace and go inside it
WORKDIR /build
RUN cargo new application
WORKDIR /build/application

# Download and compile Rust dependencies (and store as a separate Docker layer)
COPY Cargo.toml Cargo.lock ./
RUN cargo build --target "$(uname -m)-unknown-linux-musl" --release

# Build the executable using the actual source code
COPY src ./src
RUN touch src/main.rs \
    && cargo install --target "$(uname -m)-unknown-linux-musl" --path .

# Copy the executable to a known place
RUN cp /usr/local/cargo/bin/${PROJECT} ./built

# Create an empty Docker image
FROM scratch

# Copy built binary from the builder
COPY --from=builder /build/application/built /app

EXPOSE 8000
USER 1000

ENTRYPOINT ["/app"]

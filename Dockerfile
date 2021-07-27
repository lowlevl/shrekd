# Start the `builder` image with the desired rustc version
FROM rustlang/rust:nightly-slim as builder

WORKDIR /usr/src
ARG NAME="shrt"

# Compute the $PLATFORM variable from the running architechture
RUN export PLATFORM="$(uname -m)-unknown-linux-musl"

# Prepare the image for static linking
RUN apt-get update \
    && apt-get dist-upgrade -y \
    && apt-get install -y musl-tools \
    && rustup target add "${PLATFORM}"

# Create the application workspace
RUN USER=root cargo new application
WORKDIR /usr/src/application

# Download and compile Rust dependencies (and store as a separate Docker layer)
COPY Cargo.toml Cargo.lock ./
RUN cargo build --target "${PLATFORM}" --release

# Build the executable using the actual source code
COPY src ./src
RUN touch src/main.rs \
    && cargo install --target "${PLATFORM}" --path .

# Copy the executable to a known place
RUN cp /usr/local/cargo/bin/${NAME} ./built

# Create an empty Docker image
FROM scratch

# Copy built binary from the builder
COPY --from=builder /usr/src/application/built /app

EXPOSE 8000
USER 1000

ENTRYPOINT ["/app"]

# Create an empty Docker image
FROM scratch

# Copy built binary from the builder
COPY --chmod=755 ./built /project

# Expose the server's port
EXPOSE 8000

ENTRYPOINT ["/project"]

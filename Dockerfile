# Create an empty Docker image
FROM scratch

# Copy binary from the local directory
COPY --chmod=755 ./built /project

# Copy the UI files into the container
COPY --chmod=644 ./ui /ui

# Expose the server's port
EXPOSE 8000

ENTRYPOINT ["/project"]

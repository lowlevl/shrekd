# Create an empty Docker image
FROM scratch

USER 10099:10099

# Copy binary from the local directory
COPY --chmod=555 ./built /project

# Copy the UI files into the container
COPY --chmod=444 ./ui /ui

# Expose the server's port
EXPOSE 8000

ENTRYPOINT ["/project"]

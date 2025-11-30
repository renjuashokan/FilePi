# Use official Debian stable (e.g., bookworm)
FROM debian:bookworm-slim

# Install only Debian packaging tools (no Rust or .NET build tools)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        dpkg-dev \
        fakeroot \
        lintian \
        file && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user matching the host UID (1000)
RUN useradd -m -u 1000 builder

USER builder
WORKDIR /build

# Default command to run the build script
CMD ["./build-deb.sh"]
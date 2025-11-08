# Use the latest Fedora base image
FROM registry.fedoraproject.org/fedora:latest

# Install all development dependencies
RUN dnf update -y && \
    dnf install -y \
    gcc \
    make \
    pkg-config \
    gtk4-devel \
    libadwaita-devel \
    curl \
    openssl-devel \
    pre-commit \
    git \
    flatpak \
    flatpak-builder \
    meson \
    python3-pip \
    python3-gobject \
    itstool \
    desktop-file-utils \
    && dnf clean all

# Install rustup (the recommended Rust installer)
# We run this in a non-interactive way using the -y flag
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Configure Git
RUN git config --system safe.directory '*'

# Add Cargo's bin directory to the system's PATH
# This makes `rustc`, `cargo`, etc. available in the shell
ENV PATH="/root/.cargo/bin:${PATH}"

# Set a default command to run when the container starts
CMD ["/bin/bash"]

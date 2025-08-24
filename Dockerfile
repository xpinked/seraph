# Use the official Rust image as a base image
FROM rust:latest

# Set the working directory
WORKDIR /app

# RUN apt update && \
#     apt install -y \
#     ca-certificates \
#     curl
# RUN install -m 0755 -d /etc/apt/keyrings
# RUN curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
# RUN chmod a+r /etc/apt/keyrings/docker.asc
# RUN echo \
#     "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
#     $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}") stable" | \
#     sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
# RUN apt-get update
# RUN apt install install -y \
#     docker-ce \ 
#     docker-ce-cli \
#     containerd.io \
#     docker-buildx-plugin \
#     docker-compose-plugin

# Copy the Cargo.toml and Cargo.lock files
COPY seraph_backend/ ./seraph_backend
COPY seraph_core/ ./seraph_core
COPY seraph_workers/ ./seraph_workers
COPY migration/ ./migration
COPY Cargo.toml .

RUN cargo install sea-orm-cli

# Copy the source code
COPY src ./src

# Build the backend
RUN cargo build --release --bin seraph

# Build workers
RUN cd ./seraph_workers/ && cargo build --release --bin code_nodes_consumer && cd ..



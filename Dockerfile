# Use the official Rust image as a base image
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml .

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Set the command to run the application
CMD ["./target/release/seraph"]

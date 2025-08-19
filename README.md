# Seraph

Seraph is a project designed to manage and execute code nodes using a worker-based architecture. It leverages Rust's asynchronous capabilities and Docker for containerized execution of tasks.

## Table of Contents
- [Setup](#setup)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Running the Project](#running-the-project)
- [Debugging Locally](#debugging-locally)
- [Contributing](#contributing)

---

## Setup

### Prerequisites

Before setting up the project, ensure you have the following installed:

1. **Rust**: Install Rust using [rustup](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Docker**: Install Docker by following the instructions for your platform on the [Docker website](https://www.docker.com/get-started).

3. **Docker Compose**: Ensure Docker Compose is installed. It is often bundled with Docker Desktop or can be installed separately.

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/xpinked/seraph.git
   cd seraph
   ```

2. Build and start the services using Docker Compose:
   ```bash
   docker-compose -f docker.env up --build -d
   ```

   This will set up all dependencies and services required for local development.

---

## Running the Project

1. Start the backend server:
   ```bash
   cargo run --bin seraph_backend
   ```

2. The server will be available at `http://127.0.0.1:8080` (or the address specified in your `.env` file).

3. Use tools like `curl` or Postman to interact with the API endpoints.

---

## Debugging Locally

### Using `RUST_LOG`

1. Enable detailed logging by setting the `RUST_LOG` environment variable:
   ```bash
   export RUST_LOG=debug
   ```

2. Run the server with logging enabled:
   ```bash
   cargo run --bin seraph_backend
   ```

3. Logs will be displayed in the terminal, providing insights into the server's behavior.

### Debugging Worker Threads

1. Add tracing logs in the `worker.rs` file to monitor task execution.
2. Use `tokio-console` for visualizing asynchronous tasks:
   - Add `tokio-console` as a dependency in `Cargo.toml`:
     ```toml
     [dependencies]
     tokio-console = "0.1"
     ```
   - Run the server with the console enabled:
     ```bash
     RUST_LOG=debug TOKIO_CONSOLE_BIND=127.0.0.1:6669 cargo run --bin seraph_backend
     ```
   - Open `http://127.0.0.1:6669` in your browser to view the console.

### Debugging Docker Tasks

1. Use Docker logs to debug container execution:
   ```bash
   docker logs <container_id>
   ```

2. Ensure the Docker daemon is running and accessible.

3. Verify container creation and execution using the Docker CLI:
   ```bash
   docker ps -a
   ```

---

## Contributing

We welcome contributions to Seraph! To contribute:

1. Fork the repository and create a new branch for your feature or bugfix.
2. Make your changes and ensure the code is well-documented.
3. Run tests to verify your changes:
   ```bash
   cargo test
   ```
4. Submit a pull request with a detailed description of your changes.

### Code Style

- Follow Rust's standard formatting guidelines.
- Use `cargo fmt` to format your code.
- Use `cargo clippy` to lint your code and fix any warnings.

### Reporting Issues

If you encounter any issues, please open an issue on the [GitHub repository](https://github.com/xpinked/seraph/issues) with detailed information about the problem.

---

Thank you for contributing to Seraph! Together, we can make this project even better.

# Seraph

Seraph is a project designed to manage and execute code nodes using a worker-based architecture. It leverages Rust's asynchronous capabilities and Docker for containerized execution of tasks.

## Table of Contents

- [Setup](#setup)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Running the Project](#running-the-project)
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

3. Building dependencies images

The project has custom images dependencies for each language runtime.

### Build dependencies images

Build these images before running the project,
These images are the runtimes of specific languages.

- Python

```bash
docker build -t seraph_python -f docker/seraph_python/Dockerfile .
```

---

## Running the Project

1. Start the backend server:

   ```bash
   cargo run --bin seraph_backend
   ```

2. The server will be available at `http://127.0.0.1:8080` (or the address specified in your `.env` file).

3. Use tools like `curl` or Postman to interact with the API endpoints.

### Running migrations

The project is using Sea ORM as the SQL migration manager,

Either within the docker container or in local setup apply this steps.

1. Make sure you have sea-orm-cli installed:
   ```bash
   cargo install sea-orm-cli
   ```
2. Check migration status
   ```bash
   sea-orm-cli migrate status
   ```
3. Apply pending migrations
   ```bash
   sea-orm-cli migrate up
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

# Build stage
FROM rust:1.82 AS builder

# Set the working directory
WORKDIR /app

# Copy Cargo files to leverage Docker cache
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage (small final image)
FROM debian:bullseye-slim

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/migrator .

# Set the binary as the entrypoint
CMD ["./migrator"]


# FROM rust:1.82

# COPY . .
# RUN cargo build
# CMD ["cargo", "run"]

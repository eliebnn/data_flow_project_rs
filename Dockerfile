# Start from the official Rust image to ensure we have the latest versions of Rust and Cargo
FROM rust:1.67 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin data_flow_project_rs
WORKDIR /data_flow_project_rs

# Copy over your source code
COPY ./src ./src
COPY ./.env ./.env
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# This step will cache dependencies - thus, will only be re-run when your dependencies change
RUN cargo build --release
RUN rm src/*.rs

# Build your project
COPY ./src ./src
RUN cargo build --release

# Our final image starts here
FROM debian:bullseye-slim

# Install OpenSSL and CA certificates
RUN apt-get update && apt-get install -y openssl libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the .env file into our workdir
COPY ./.env ./.env

# Copy the build artifact from the builder stage
COPY --from=builder /data_flow_project_rs/target/release/data_flow_project_rs .

EXPOSE 8094

# Set the startup command to run your binary
CMD ["./data_flow_project_rs"]
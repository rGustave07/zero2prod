# Build Stage
FROM lukemathwalker/cargo-chef:latest-rust-1.74.1 as chef

WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
# this layer creates the lock file so that
# we may check to see if this generated file has a diff
# If not then we can use the cached dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
ENV SQLX_OFFLINE true
# Build our project
RUN cargo build --release --bin zero2prod

# Runtime Stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates (For HTTPS)
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# Copy from the builder, the built binary to run application to our runtime stage
# Also copy config
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
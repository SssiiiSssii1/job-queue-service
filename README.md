# job-queue-service

A Rust-based GraphQL job queue that processes background tasks asynchronously using Tokio, with concurrent state management, automatic retries, job tracking, and graceful shutdown.

## Stack

- [Axum](https://github.com/tokio-rs/axum) for the HTTP server
- [async-graphql](https://github.com/async-graphql/async-graphql) for the GraphQL schema
- [Tokio](https://tokio.rs) for the async runtime and worker task
- UUIDs for job IDs

## How it works

- Jobs are pushed onto a bounded queue (capacity 100).
- A background worker takes jobs from the queue and processes them.
- Job state is kept in a shared in-memory map, so jobs can be tracked by ID through the API.
- On Ctrl+C the server stops accepting requests, tells the worker to shut down, and waits for it to finish.

## Requirements

- Rust toolchain with edition 2024 support

## Run

    cargo run

The service listens on `http://0.0.0.0:3000` and serves GraphQL at `POST /graphql`.

## Project layout

- `src/main.rs` starts the server, the worker and the shutdown handling
- `src/graphql` holds the query and mutation schema
- `src/queue` holds the job queue
- `src/workers` holds the worker that processes jobs
- `src/models` holds the job model

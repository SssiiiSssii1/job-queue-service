# Job Queue Service

A small, professional **GraphQL job queue service** written in Rust. It accepts jobs
over GraphQL, stores them in an in-memory queue, and processes them asynchronously
with background workers — including automatic retries and graceful shutdown.

## What it does

- Accept job submissions via a GraphQL `createJob` mutation
- Store jobs in memory and track their status lifecycle
- Process jobs in the background with a Tokio worker
- Retry failed jobs up to **3 times**
- Query jobs by ID or list all jobs
- Shut down gracefully without losing in-flight work

## Tech stack

| Layer      | Choice                           |
| ---------- | -------------------------------- |
| HTTP       | [Axum](https://github.com/tokio-rs/axum) |
| GraphQL    | [async-graphql](https://github.com/async-graphql/async-graphql) |
| Runtime    | [Tokio](https://tokio.rs) |
| Serialization | [Serde](https://serde.rs) |
| IDs        | [UUID](https://github.com/uuid-rs/uuid) |

No database, Redis, Kafka, authentication, or external services — everything lives
in memory.

## Architecture

```text
src/
├── main.rs            # Server bootstrap, routing, graceful shutdown
├── models/
│   └── job.rs         # Job model and JobStatus enum
├── queue.rs           # In-memory queue (tokio mpsc channel)
├── graphql/
│   └── schema.rs      # GraphQL types, resolvers, shared AppState
└── workers.rs         # Background worker with retry logic
```

Shared application state is an `AppState` struct containing:

- `jobs: Arc<Mutex<HashMap<Uuid, Job>>>` — the job store queried by GraphQL
- `queue: mpsc::Sender<Job>` — the queue the worker consumes from

State is registered in the GraphQL schema so resolvers can access it safely from any
task via `ctx.data::<AppState>()`.

### How the worker works

1. `createJob` inserts the job into the store and pushes it onto the channel.
2. The worker awaits messages on the channel with `tokio::select!`.
3. On receipt, the job transitions `PENDING → PROCESSING`.
4. The worker runs the job handler (a simulated async task) and then a retry loop.
5. The final status is written back to the shared store: `COMPLETED` or `FAILED`.

### Retry behavior

- A job is attempted up to **3 times** (`MAX_RETRIES`).
- Between attempts, the job stays `PROCESSING`; `attempts` is incremented each try.
- If all 3 attempts fail, the job is marked `FAILED`.

The worker includes deterministic job types for demonstration:

| Job type     | Behavior                                  |
| ------------ | ----------------------------------------- |
| `send_email` | Succeeds on the first attempt             |
| `flaky`      | Fails once, succeeds on retry (2 attempts)|
| `failing`    | Always fails — exhausts all 3 retries     |

## Getting started

### Requirements

- Rust toolchain (edition 2024)

### Run

```sh
cargo run
```

The server listens on `http://localhost:3000`, GraphQL endpoint at
`POST /graphql`.

## GraphQL schema

```graphql
type Job {
  id: ID!
  type: String!
  payload: String!
  status: JobStatus!
  attempts: Int!
}

enum JobStatus {
  PENDING
  PROCESSING
  COMPLETED
  FAILED
}

type Query {
  job(id: ID!): Job
  jobs: [Job!]!
}

type Mutation {
  createJob(type: String!, payload: String!): Job!
}
```

## Example queries

Create a job:

```graphql
mutation {
  createJob(
    type: "send_email"
    payload: "hello@example.com"
  ) {
    id
    type
    status
    attempts
  }
}
```

List all jobs:

```graphql
query {
  jobs {
    id
    type
    payload
    status
    attempts
  }
}
```

Get a single job:

```graphql
query {
  job(id: "062df345-2bf2-4414-8563-2fbd96f1b8bd") {
    type
    payload
    status
    attempts
  }
}
```

### cURL

```sh
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query":"mutation { createJob(type: \"send_email\", payload: \"hi@example.com\") { id status } }"}'
```

## Graceful shutdown

Press `Ctrl+C` (SIGINT). The server stops accepting requests, signals the worker to
stop, wait for it to finish any in-flight job, and exits cleanly.

## Notes

- This is intentionally simple: a single in-memory queue, one worker, no
  persistence. Swap the `models` and `queue` modules for a real store (Postgres,
  Redis) when scaling up.
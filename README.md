# Concurrent Job Processing System

A backend job processing system built with **Go** and **NestJS**, using **gRPC**, **channels**, and a **worker pool** for concurrent job processing.

## Architecture

```text
NestJS REST API
      ↓
    gRPC
      ↓
Go Job Service
      ↓
Buffered Channel
      ↓
Worker Pool
      ↓
Job Processing
```

## Tech Stack

* Go
* NestJS
* gRPC / Protobuf
* Goroutines
* Channels
* Worker Pool
* Sync Mutex / WaitGroup

## Features

* Submit jobs
* Track job status
* Concurrent job processing
* Worker pool
* In-memory job storage
* gRPC communication

## Run

### Go

```bash
go run ./cmd/server
```

### NestJS

```bash
cd nestjs-api
pnpm install
pnpm start:dev
```

Go service runs on:

```text
localhost:50051
```

NestJS API runs on:

```text
localhost:3000
```

## API

### Submit Job

```http
POST /jobs
```

```json
{
  "type": "email",
  "payload": "hello"
}
```

### Get Job

```http
GET /jobs/:id
```

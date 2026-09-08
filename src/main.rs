use axum::Router;
use axum::routing::post;
use async_graphql::EmptySubscription;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use crate::graphql::schema::{Query, Mutation, AppState};
use crate::queue::create_queue;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{watch, Mutex};

type AppSchema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

#[tokio::main]
async fn main() {
    let (queue, receiver) = create_queue(100);
    let jobs: Arc<Mutex<HashMap<uuid::Uuid, crate::models::job::Job>>> = Arc::new(Mutex::new(HashMap::new()));

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let jobs_clone = Arc::clone(&jobs);
    let worker_handle = tokio::spawn(async move {
        crate::workers::worker("worker-1".to_string(), receiver, jobs_clone, shutdown_rx).await;
    });

    let app_state = AppState {
        jobs: Arc::clone(&jobs),
        queue,
    };

    let schema = async_graphql::Schema::build(Query, Mutation, EmptySubscription)
        .data(app_state)
        .finish();

    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .with_state(Arc::new(schema));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.expect("failed to bind");
    println!("Job Queue Service running at http://{}", listener.local_addr().unwrap());

    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(err) = result {
                eprintln!("Server error: {err}");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Shutdown signal received, stopping gracefully...");
        }
    }

    let _ = shutdown_tx.send(true);
    worker_handle.await.expect("worker panicked");
    println!("Shutdown complete");
}

async fn graphql_handler(
    axum::extract::State(schema): axum::extract::State<Arc<AppSchema>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

mod models;
mod queue;
mod workers;
mod graphql;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};
use crate::models::job::{Job, JobStatus};

pub const MAX_RETRIES: usize = 3;

pub async fn worker(
    name: String,
    mut receiver: mpsc::Receiver<Job>,
    jobs: Arc<Mutex<HashMap<uuid::Uuid, Job>>>,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            job = receiver.recv() => {
                let Some(job) = job else { break };
                eprintln!("Worker {} received job {}", name, job.id);
                process_job_with_retries(&name, job, &jobs).await;
            }
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    eprintln!("Worker {} shutting down...", name);
                    break;
                }
            }
        }
    }

    eprintln!("Worker {} stopped", name);
}

async fn process_job_with_retries(
    name: &str,
    job: Job,
    jobs: &Arc<Mutex<HashMap<uuid::Uuid, Job>>>,
) {
    let mut attempts = 0;

    loop {
        attempts += 1;
        {
            let mut jobs = jobs.lock().await;
            if let Some(entry) = jobs.get_mut(&job.id) {
                entry.status = JobStatus::PROCESSING;
                entry.attempts = attempts;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        match process_job(&job, attempts) {
            Ok(()) => {
                let mut jobs = jobs.lock().await;
                if let Some(entry) = jobs.get_mut(&job.id) {
                    entry.status = JobStatus::COMPLETED;
                    eprintln!("Worker {} job {} completed after {} attempt(s)", name, job.id, attempts);
                }
                break;
            }
            Err(err) => {
                if attempts >= MAX_RETRIES {
                    let mut jobs = jobs.lock().await;
                    if let Some(entry) = jobs.get_mut(&job.id) {
                        entry.status = JobStatus::FAILED;
                        entry.attempts = attempts;
                        eprintln!("Worker {} job {} failed after {} attempts: {}", name, job.id, attempts, err);
                    }
                    break;
                }
                eprintln!("Worker {} job {} attempt {} failed ({}), retrying...", name, job.id, attempts, err);
            }
        }
    }
}

fn process_job(job: &Job, attempts: usize) -> Result<(), String> {
    match job.r#type.as_str() {
        "flaky" => {
            if attempts >= 2 {
                Ok(())
            } else {
                Err("temporary failure".to_string())
            }
        }
        "failing" => Err("permanent failure".to_string()),
        _ => Ok(()),
    }
}
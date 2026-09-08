use async_graphql::{Object, SimpleObject};
use crate::models::job::{Job as JobModel, JobStatus as JobStatusModel};
use crate::queue::JobSender;
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct AppState {
    pub jobs: Arc<Mutex<HashMap<uuid::Uuid, JobModel>>>,
    pub queue: JobSender,
}

pub struct Query;

#[Object]
impl Query {
    async fn job(&self, ctx: &async_graphql::Context<'_>, id: String) -> Option<Job> {
        let data = ctx.data::<AppState>().unwrap();
        let jobs = data.jobs.lock().await;
        jobs.get(&uuid::Uuid::parse_str(&id).ok()?).cloned().map(Job::from)
    }

    async fn jobs(&self, ctx: &async_graphql::Context<'_>) -> Vec<Job> {
        let data = ctx.data::<AppState>().unwrap();
        let jobs = data.jobs.lock().await;
        jobs.values().cloned().map(Job::from).collect()
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_job(
        &self,
        ctx: &async_graphql::Context<'_>,
        r#type: String,
        payload: String,
    ) -> Job {
        let data = ctx.data::<AppState>().unwrap();
        let job = JobModel::new(r#type, payload);
        let id = job.id;
        let mut jobs = data.jobs.lock().await;
        jobs.insert(id, job.clone());
        drop(jobs);
        let _ = data.queue.send(job.clone()).await;
        Job::from(job)
    }
}

#[derive(SimpleObject, Clone)]
pub struct Job {
    id: String,
    r#type: String,
    payload: String,
    status: JobStatus,
    attempts: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, async_graphql::Enum)]
pub enum JobStatus {
    PENDING,
    PROCESSING,
    COMPLETED,
    FAILED,
}

impl From<JobModel> for Job {
    fn from(job: JobModel) -> Self {
        Job {
            id: job.id.to_string(),
            r#type: job.r#type,
            payload: job.payload,
            status: match job.status {
                JobStatusModel::PENDING => JobStatus::PENDING,
                JobStatusModel::PROCESSING => JobStatus::PROCESSING,
                JobStatusModel::COMPLETED => JobStatus::COMPLETED,
                JobStatusModel::FAILED => JobStatus::FAILED,
            },
            attempts: job.attempts,
        }
    }
}

use tokio::sync::mpsc;
use crate::models::job::Job;

pub type JobSender = mpsc::Sender<Job>;

pub fn create_queue(capacity: usize) -> (JobSender, mpsc::Receiver<Job>) {
    let (sender, receiver) = mpsc::channel(capacity);
    (sender, receiver)
}

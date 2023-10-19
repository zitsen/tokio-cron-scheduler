use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
    let sched = JobScheduler::new().await;
    let sched = sched.unwrap();

    let job_run_times = Arc::new(AtomicUsize::new(0));
    let job_run_times_in_job = job_run_times.clone();
    let job = Job::new_repeated_async(Duration::from_secs(2), move |_uuid, mut _l| {
        let job_run_times_in_job = job_run_times_in_job.clone();
        Box::pin(async move {
            info!("I'm repeated async every 4 seconds");
            tokio::time::sleep(Duration::from_secs(7)).await;
            info!("I'm repeated async every 4 seconds but we sleep 7 seconds");
            job_run_times_in_job.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
            match _l.next_tick_for_job(_uuid).await {
                Ok(val) => {
                    info!("Next tick for job {:?} is {:?}", _uuid, val);
                }
                Err(e) => {
                    info!("Error getting next tick for job {:?} {:?}", _uuid, e);
                }
            }
        })
    })
    .unwrap();

    sched.add(job).await.unwrap();
    sched.start().await.unwrap();
    tokio::time::sleep(Duration::from_secs(20)).await;
    let times = job_run_times.load(Ordering::Acquire);
    assert!(times == 2);
}


use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use bimap::BiMap;
use tokio::sync::RwLock;
use tokio_cron_scheduler::JobScheduler;
use tracing::info;
use uuid::Uuid;

type RunningJobIds = BiMap<i32, Uuid>;

pub struct Scheduler {
    internal: JobScheduler,
    running_job_ids: Arc<RwLock<RunningJobIds>>,
    stopping: Mutex<bool>,
}

impl Scheduler {
    pub async fn new() -> Result<Scheduler, Box<dyn Error>> {
        let mut scheduler = JobScheduler::new().await?;

        #[cfg(feature = "signal")]
        scheduler.shutdown_on_ctrl_c();

        #[cfg(feature = "signal")]
        scheduler.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("Scheduler Shutdown done");
            })
        }));

        Ok(Scheduler {
            internal: scheduler,
            running_job_ids: Arc::new(Default::default()),
            stopping: Mutex::new(false),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        self.internal.start().await?;
        Ok(())
    }

    pub async fn job_exist(&self, id: i32) -> bool {
        let jobs = self.running_job_ids.read().await;
        let job_id = jobs.get_by_left(&id);
        job_id.is_some()
    }

}

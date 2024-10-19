use crate::scheduling::Task;
use application_beans::factory::bean_factory::BeanFactory;
use application_context::context::application_context::APPLICATION_CONTEXT;
use application_core::lang::runnable::Runnable;
use bimap::BiMap;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_cron_scheduler::{JobBuilder, JobScheduler, JobSchedulerError};
use tracing::info;
use uuid::Uuid;

type RunningJobIds = BiMap<i32, Uuid>;

pub struct Scheduler {
    internal: JobScheduler,
    job_ids: Arc<RwLock<RunningJobIds>>,
    tasks: Arc<RwLock<HashMap<Uuid, Task>>>,
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
            job_ids: Arc::new(Default::default()),
            tasks: Arc::new(Default::default()),
            stopping: Mutex::new(false),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        self.internal.start().await?;
        Ok(())
    }

    pub async fn job_exist(&self, id: i32) -> bool {
        let jobs = self.job_ids.read().await;
        let job_id = jobs.get_by_left(&id);
        job_id.is_some()
    }

    async fn add_cron_task(&self, cron: &str, task: Task) -> Result<Uuid, JobSchedulerError> {
        let jj = JobBuilder::new()
            .with_timezone(chrono_tz::Asia::Shanghai)
            .with_cron_job_type()
            .with_schedule(cron)
            .unwrap()
            .with_run_async(Box::new(|uuid, _locked| {
                Box::pin(async move {
                    let application_context = APPLICATION_CONTEXT.read().await;
                    let scheduler = application_context.get_bean_factory().get::<Scheduler>();
                    let stopping = scheduler.stopping.lock().await;
                    if *stopping {
                        // scheduler is stopping, just return
                        return;
                    }
                    let tasks = scheduler.tasks.read().await;
                    let task = tasks.get(&uuid);
                    if let Some(task) = task {
                        task.get_runnable().run().await;
                    }
                })
            }))
            .build()?;

        let result = self.internal.add(jj).await;
        match result {
            Ok(uuid) => {
                let mut tasks = self.tasks.write().await;
                tasks.insert(uuid, task);
                Ok(uuid)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn add_job(
        &self,
        id: i32,
        name: &str,
        cron: &str,
        runnable: Box<dyn Runnable>,
    ) -> Result<(), Box<dyn Error>> {
        match self.add_cron_task(cron, Task::new(runnable)).await {
            Ok(uuid) => {
                let mut jobs = self.job_ids.write().await;
                jobs.insert(id, uuid);

                info!("Scheduler add job {} cron {}", name, cron);
            }
            Err(_) => info!("Scheduler add job {} failed.", name),
        }
        Ok(())
    }

    pub async fn reload_job(
        &self,
        id: i32,
        name: &str,
        cron: &str,
        runnable: Box<dyn Runnable>,
    ) -> Result<(), Box<dyn Error>> {
        self.stop_job(id).await?;

        self.add_job(id, name, cron, runnable).await?;

        Ok(())
    }

    pub async fn stop_job(&self, id: i32) -> Result<(), Box<dyn Error>> {
        let mut job_ids = self.job_ids.write().await;

        let job_id = job_ids.get_by_left(&id);
        match job_id {
            Some(uuid) => {
                self.internal.remove(uuid).await?;
                let mut tasks = self.tasks.write().await;
                tasks.remove(uuid);
                job_ids.remove_by_left(&id);
                Ok(())
            }
            None => Err("Job id not found".into()),
        }
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn Error>> {
        {
            let mut stopping = self.stopping.lock().await;
            *stopping = true;
        }
        let job_ids = self.get_job_ids().await;

        for job_id in job_ids {
            let _ = self.stop_job(job_id).await;
        }

        Ok(())
    }

    pub async fn get_job_ids(&self) -> Vec<i32> {
        let running_jobs = self.job_ids.read().await;
        let iter = running_jobs.iter();
        let mut job_ids = Vec::new();
        for (id, _uuid) in iter {
            job_ids.push(*id);
        }
        job_ids
    }
}

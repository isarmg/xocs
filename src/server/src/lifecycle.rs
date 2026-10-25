use blog_application::server_runtime::{LifecycleParticipant, WorkScope};
use sqlx::SqlitePool;
use std::sync::Mutex;

pub struct Lifecycle {
    pub scope: WorkScope,
    pub pool: SqlitePool,
    pub lock: Mutex<Option<std::sync::Arc<blog_application::state_file::InstanceLock>>>,
}

#[async_trait::async_trait]
impl LifecycleParticipant for Lifecycle {
    fn quiesce(&self) {
        tracing::info!(event = "common.runtime.shutdown_started");
        self.scope.quiesce();
    }
    fn cancel_ordinary_work(&self) {
        self.scope.cancel_ordinary_work();
    }
    fn active_tasks(&self) -> (usize, usize) {
        (self.scope.work_tasks.len(), self.scope.commit_tasks.len())
    }
    async fn drain_requests(&self) {
        self.scope.drain_requests().await;
    }
    async fn drain_commits(&self) {
        self.scope.drain_commits().await;
    }
    async fn close_state(&self) -> Result<(), String> {
        self.pool.close().await;
        self.lock
            .lock()
            .expect("instance state lock poisoned")
            .take();
        Ok(())
    }
}

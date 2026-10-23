//! Fair permits for connection limits. The product uses Tokio throughout.
pub struct AsyncSemaphore {
    inner: tokio::sync::Semaphore,
}

impl AsyncSemaphore {
    pub fn new(fair: bool, permits: usize) -> Self {
        debug_assert!(fair, "connection pools require fair permits");
        Self {
            inner: tokio::sync::Semaphore::new(permits),
        }
    }

    pub fn permits(&self) -> usize {
        self.inner.available_permits()
    }

    pub async fn acquire(&self, permits: u32) -> AsyncSemaphoreReleaser<'_> {
        AsyncSemaphoreReleaser {
            inner: self
                .inner
                .acquire_many(permits)
                .await
                .expect("the pool never closes its semaphore"),
        }
    }

    pub fn try_acquire(&self, permits: u32) -> Option<AsyncSemaphoreReleaser<'_>> {
        Some(AsyncSemaphoreReleaser {
            inner: self.inner.try_acquire_many(permits).ok()?,
        })
    }

    pub fn release(&self, permits: usize) {
        self.inner.add_permits(permits);
    }
}

pub struct AsyncSemaphoreReleaser<'a> {
    inner: tokio::sync::SemaphorePermit<'a>,
}

impl AsyncSemaphoreReleaser<'_> {
    pub fn disarm(self) {
        self.inner.forget();
    }
}

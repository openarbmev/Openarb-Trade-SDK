use tokio::task::JoinHandle;

pub struct SubscriptionHandle {
    pub task: JoinHandle<()>,
    pub unsub_fn: Box<dyn Fn() + Send>,
}

impl SubscriptionHandle {
    pub async fn shutdown(self) {
        (self.unsub_fn)();
        self.task.abort();
    }
}

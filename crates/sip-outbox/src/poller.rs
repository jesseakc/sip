use tracing::info;

pub struct OutboxPoller;

impl Default for OutboxPoller {
    fn default() -> Self {
        Self::new()
    }
}

impl OutboxPoller {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self) {
        info!("Outbox poller started");
    }
}

use tracing::info;

pub struct OutboxPoller;

impl OutboxPoller {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self) {
        info!("Outbox poller started");
    }
}

mod rate_limited_queue;

use std::time::Duration;

use interface_types::Telemetry;
use tokio_util::sync::CancellationToken;

use crate::domain::rate_limited_queue::{QueueConfig, RateLimitedQueue};
use crate::infrastructure::config::Config;
use crate::interface::{LogWriter, TelemetryReceiver};

pub struct TelemetryService<R: TelemetryReceiver, W: LogWriter<Telemetry>> {
    receiver: R,
    writer: W,
    deque: RateLimitedQueue<Telemetry>,
    config: Config,
    full_queue_notifier: tokio::sync::mpsc::Receiver<()>,
}

impl<R: TelemetryReceiver, W: LogWriter<Telemetry>> TelemetryService<R, W> {
    pub fn new(receiver: R, writer: W, config: Config) -> Self {
        let queue_config = QueueConfig {
            max_capacity: config.buffer_size,
            rate_limit: config.rate_limit,
            time_window: Duration::from_secs(1),
        };
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let deque = RateLimitedQueue::new(queue_config, tx);
        Self {
            receiver,
            writer,
            deque,
            config,
            full_queue_notifier: rx,
        }
    }

    pub async fn run(&mut self, cancellation_token: CancellationToken) {
        let mut flush = tokio::time::interval(self.config.buffer_flush_interval);

        loop {
            tokio::select! {
                message = self.receiver.recv() => {
                    match message {
                        Some(message) => {
                            self.try_enqueue_telemetry(message).await;
                        }
                        None => {
                            tracing::error!("Telemetry sender closed");
                            break;
                        }
                    }
                }
                _ = flush.tick() => {
                    self.flush().await;
                }
                Some(()) = self.full_queue_notifier.recv() => {
                    self.flush().await;
                }
                _ = cancellation_token.cancelled() => {
                    tracing::info!("Telemetry service shutting down");
                    break;
                }
            }
        }
    }

    async fn try_enqueue_telemetry(&mut self, message: Telemetry) {
        let _ = self.deque.enqueue(message).await;
    }

    async fn flush(&mut self) {
        let items = self.deque.dequeue_all();
        items.into_iter().for_each(|item| {
            let _ = self.writer.log(item);
        })
    }
}

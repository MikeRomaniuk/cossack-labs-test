use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

/// Configuration for the rate-limited queue
pub struct QueueConfig {
    /// Maximum number of items the queue can hold
    pub max_capacity: usize,
    /// Maximum number of items that can be processed per time window
    pub rate_limit: usize,
    /// Time window for rate limiting
    pub time_window: Duration,
}

/// A fixed-size queue with rate limiting that drops requests exceeding the limit
pub struct RateLimitedQueue<T> {
    queue: VecDeque<T>,
    config: QueueConfig,
    rate_tracker: RateTracker,
    full_notifier: mpsc::Sender<()>,
}

struct RateTracker {
    requests: VecDeque<Instant>,
    window: Duration,
    limit: usize,
}

impl RateTracker {
    fn new(limit: usize, window: Duration) -> Self {
        Self {
            requests: VecDeque::new(),
            window,
            limit,
        }
    }

    /// Check if we can accept a new request within rate limits
    fn can_accept(&mut self) -> bool {
        let now = Instant::now();

        // Remove timestamps outside the current window
        while let Some(&oldest) = self.requests.front() {
            if now.duration_since(oldest) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Check if we're under the rate limit
        self.requests.len() < self.limit
    }

    /// Record a new request timestamp
    fn record_request(&mut self) {
        self.requests.push_back(Instant::now());
    }

    /// Get current request count in the window
    fn current_count(&mut self) -> usize {
        let now = Instant::now();

        // Clean up old requests
        while let Some(&oldest) = self.requests.front() {
            if now.duration_since(oldest) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        self.requests.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnqueueResult {
    /// Item was successfully enqueued
    Success,
    /// Item was dropped because queue is full
    QueueFull,
    /// Item was dropped because rate limit exceeded
    RateLimitExceeded,
}

impl<T> RateLimitedQueue<T> {
    /// Create a new rate-limited queue with the given configuration
    pub fn new(config: QueueConfig, notifier: mpsc::Sender<()>) -> Self {
        Self {
            queue: VecDeque::with_capacity(config.max_capacity),
            rate_tracker: RateTracker::new(config.rate_limit, config.time_window),
            config,
            full_notifier: notifier,
        }
    }

    /// Attempt to enqueue an item
    /// Returns EnqueueResult indicating whether the item was accepted or dropped
    pub async fn enqueue(&mut self, item: T) -> EnqueueResult {
        // Check rate limit first
        if !self.rate_tracker.can_accept() {
            return EnqueueResult::RateLimitExceeded;
        }

        // Check queue capacity
        if self.queue.len() >= self.config.max_capacity {
            // Trigger full callback if set
            let _ = self.full_notifier.send(()).await;
            return EnqueueResult::QueueFull;
        }

        // Accept the item
        self.queue.push_back(item);
        self.rate_tracker.record_request();

        EnqueueResult::Success
    }

    /// Dequeue an item from the front of the queue
    pub fn dequeue(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    /// Get the current number of items in the queue
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Check if the queue is full
    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.config.max_capacity
    }

    /// Get the current rate limit usage (requests in current window)
    pub fn current_rate_count(&mut self) -> usize {
        self.rate_tracker.current_count()
    }

    /// Clear all items from the queue
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn dequeue_all(&mut self) -> Vec<T> {
        self.queue.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_enqueue_dequeue() {
        let config = QueueConfig {
            max_capacity: 5,
            rate_limit: 10,
            time_window: Duration::from_secs(1),
        };
        let (tx, _) = mpsc::channel(1);
        let mut queue = RateLimitedQueue::new(config, tx);

        assert_eq!(queue.enqueue(1).await, EnqueueResult::Success);
        assert_eq!(queue.enqueue(2).await, EnqueueResult::Success);
        assert_eq!(queue.len(), 2);

        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), None);
    }

    #[tokio::test]
    async fn test_queue_full() {
        let config = QueueConfig {
            max_capacity: 3,
            rate_limit: 10,
            time_window: Duration::from_secs(1),
        };
        let (tx, mut rx) = mpsc::channel(1);
        let mut queue = RateLimitedQueue::new(config, tx);

        assert_eq!(queue.enqueue(1).await, EnqueueResult::Success);
        assert_eq!(queue.enqueue(2).await, EnqueueResult::Success);
        assert_eq!(queue.enqueue(3).await, EnqueueResult::Success);
        assert_eq!(queue.enqueue(4).await, EnqueueResult::QueueFull);

        assert!(queue.is_full());
        assert_eq!(queue.len(), 3);
        assert_eq!(rx.recv().await, Some(()));
    }

    #[tokio::test]
    async fn test_rate_limit() {
        let config = QueueConfig {
            max_capacity: 100,
            rate_limit: 5,
            time_window: Duration::from_millis(100),
        };

        let (tx, _) = mpsc::channel(1);
        let mut queue = RateLimitedQueue::new(config, tx);

        // Should accept first 5 requests
        for i in 0..5 {
            assert_eq!(queue.enqueue(i).await, EnqueueResult::Success);
        }

        // 6th request should be rate limited
        assert_eq!(queue.enqueue(5).await, EnqueueResult::RateLimitExceeded);
    }

    #[tokio::test]
    async fn test_rate_limit_recovery() {
        let config = QueueConfig {
            max_capacity: 100,
            rate_limit: 2,
            time_window: Duration::from_millis(50),
        };
        let (tx, _) = mpsc::channel(1);
        let mut queue = RateLimitedQueue::new(config, tx);

        // Fill rate limit
        assert_eq!(queue.enqueue(1).await, EnqueueResult::Success);
        assert_eq!(queue.enqueue(2).await, EnqueueResult::Success);
        assert_eq!(queue.enqueue(3).await, EnqueueResult::RateLimitExceeded);

        // Wait for window to pass
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should be able to enqueue again
        assert_eq!(queue.enqueue(4).await, EnqueueResult::Success);
    }

    #[tokio::test]
    async fn test_dequeue_all() {
        let config = QueueConfig {
            max_capacity: 10,
            rate_limit: 10,
            time_window: Duration::from_secs(1),
        };
        let (tx, _) = mpsc::channel(1);
        let mut queue = RateLimitedQueue::new(config, tx);

        // Enqueue some items
        for i in 0..5 {
            queue.enqueue(i).await;
        }

        assert_eq!(queue.len(), 5);

        // Dequeue all
        let items = queue.dequeue_all();

        assert_eq!(items.len(), 5);
        assert_eq!(items, vec![0, 1, 2, 3, 4]);
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn test_dequeue_all_empty() {
        let config = QueueConfig {
            max_capacity: 10,
            rate_limit: 10,
            time_window: Duration::from_secs(1),
        };
        let (tx, _) = mpsc::channel(1);
        let mut queue = RateLimitedQueue::<i32>::new(config, tx);

        let items = queue.dequeue_all();
        assert_eq!(items.len(), 0);
        assert!(items.is_empty());
    }
}

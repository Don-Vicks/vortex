use crate::lifecycle::{FailureType, LifecycleEvent};

pub enum RetryStrategy {
    RefreshBlockhashAndRetry,
    IncreaseTipAndRetry { increase_by_percent: u64 },
    WaitForNextLeader,
    DoNotRetry,
}

pub fn decide_retry(event: &LifecycleEvent) -> RetryStrategy {
    match &event.failure {
        None => RetryStrategy::DoNotRetry,
        Some(failure) => {
            if failure.retry_count >= 3 {
                return RetryStrategy::DoNotRetry;
            }
            match failure.error_type {
                FailureType::ExpiredBlockhash => RetryStrategy::RefreshBlockhashAndRetry,
                FailureType::FeeTooLow => RetryStrategy::IncreaseTipAndRetry {
                    increase_by_percent: 20,
                },
                FailureType::LeaderSkipped => RetryStrategy::WaitForNextLeader,
                FailureType::ComputeExceeded => RetryStrategy::DoNotRetry,
                FailureType::BundleFailure => RetryStrategy::IncreaseTipAndRetry {
                    increase_by_percent: 10,
                },
                FailureType::Unknown => RetryStrategy::DoNotRetry,
            }
        }
    }
}

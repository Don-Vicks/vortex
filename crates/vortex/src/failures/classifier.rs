use crate::lifecycle::FailureType;

pub fn classify(raw_error: &str) -> FailureType {
    let error = raw_error.to_lowercase();

    if error.contains("blockhash") || error.contains("expired") {
        FailureType::ExpiredBlockhash
    } else if error.contains("insufficient fee") || error.contains("fee too low") {
        FailureType::FeeTooLow
    } else if error.contains("compute") || error.contains("exceeded") {
        FailureType::ComputeExceeded
    } else if error.contains("bundle") {
        FailureType::BundleFailure
    } else if error.contains("leader") || error.contains("skipped") {
        FailureType::LeaderSkipped
    } else {
        FailureType::Unknown
    }
}

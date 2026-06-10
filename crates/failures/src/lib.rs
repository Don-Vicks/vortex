use lifecycle::FailureType;

/// Classifies a raw error string from Solana or Jito into a structured FailureType.
pub fn classify_error(raw_error: &str) -> FailureType {
    let lower = raw_error.to_lowercase();

    if lower.contains("blockhash not found") || lower.contains("blockhash expired") || lower.contains("blockhash notfound") {
        return FailureType::ExpiredBlockhash;
    } 
    
    if lower.contains("insufficient funds") || lower.contains("fee too low") || lower.contains("insufficient lamports") || lower.contains("gas too low") {
        return FailureType::FeeTooLow;
    } 
    
    if lower.contains("exceeded compute") || lower.contains("compute budget") || lower.contains("too many instructions") || lower.contains("compute units") {
        return FailureType::ComputeExceeded;
    } 
    
    if lower.contains("instruction error") || lower.contains("signature verification failed") || lower.contains("invalid instruction") {
        return FailureType::InstructionError;
    } 
    
    if lower.contains("account in use") || lower.contains("account locked") {
        return FailureType::AccountInUse;
    } 
    
    if (lower.contains("already") && lower.contains("processed")) || lower.contains("duplicate transaction") || lower.contains("alreadyprocessed") {
        return FailureType::AlreadyProcessed;
    } 
    
    if lower.contains("node is behind") || lower.contains("skipped slot") || lower.contains("leader skipped") {
        return FailureType::NodeBehind;
    } 
    
    if lower.contains("bundle") || lower.contains("rejected") || lower.contains("dropped") {
        return FailureType::BundleFailure;
    } 

    FailureType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use lifecycle::FailureType;

    #[test]
    fn test_classify_expired_blockhash() {
        assert_eq!(classify_error("Transaction simulation failed: Blockhash not found"), FailureType::ExpiredBlockhash);
        assert_eq!(classify_error("blockhash expired"), FailureType::ExpiredBlockhash);
    }

    #[test]
    fn test_classify_fee_too_low() {
        assert_eq!(classify_error("insufficient funds for fee"), FailureType::FeeTooLow);
        assert_eq!(classify_error("Fee too low for transaction"), FailureType::FeeTooLow);
    }

    #[test]
    fn test_classify_compute_exceeded() {
        assert_eq!(classify_error("Exceeded compute budget"), FailureType::ComputeExceeded);
        assert_eq!(classify_error("Program exceeded maximum compute units"), FailureType::ComputeExceeded);
    }

    #[test]
    fn test_classify_instruction_error() {
        assert_eq!(classify_error("Instruction error: signature verification failed"), FailureType::InstructionError);
    }

    #[test]
    fn test_classify_account_in_use() {
        assert_eq!(classify_error("Account in use"), FailureType::AccountInUse);
    }

    #[test]
    fn test_classify_already_processed() {
        let err = "This transaction has already been processed";
        let classified = classify_error(err);
        assert_eq!(classified, FailureType::AlreadyProcessed);
    }

    #[test]
    fn test_classify_node_behind() {
        assert_eq!(classify_error("Node is behind by 100 slots"), FailureType::NodeBehind);
    }

    #[test]
    fn test_classify_bundle_failure() {
        assert_eq!(classify_error("Jito bundle rejected"), FailureType::BundleFailure);
    }

    #[test]
    fn test_classify_unknown() {
        assert_eq!(classify_error("some weird unknown error"), FailureType::Unknown);
    }
}

export type TxStatus = 'Pending' | 'Processed' | 'Confirmed' | 'Finalized' | 'Failed';

export type FailureType =
  | 'ExpiredBlockhash'
  | 'FeeTooLow'
  | 'ComputeExceeded'
  | 'BundleFailure'
  | 'LeaderSkipped'
  | 'Unknown';

export interface FailureInfo {
  error_type: FailureType;
  raw_error: string;
  retry_count: number;
  resolved: boolean;
}

export interface LifecycleEvent {
  id: string;
  bundle_id: string;
  signature: string;
  tip_lamports: number;
  tip_reasoning: string;
  submitted_at: string;
  submitted_slot: number;
  processed_at: string | null;
  processed_slot: number | null;
  confirmed_at: string | null;
  confirmed_slot: number | null;
  finalized_at: string | null;
  finalized_slot: number | null;
  latency_to_processed_ms: number | null;
  latency_to_confirmed_ms: number | null;
  latency_to_finalized_ms: number | null;
  status: TxStatus;
  failure: FailureInfo | null;
}

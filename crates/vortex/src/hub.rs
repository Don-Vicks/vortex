//! Fan-out point for decoded transactions coming off the Geyser stream.
//!
//! The Geyser consumer publishes every `VortexTransaction` here; any number of
//! downstream consumers subscribe. Consumers also register the programs they
//! care about, and the hub turns the union into the live stream filters.

use crate::events::VortexTransaction;
use crate::geyser::stream::StreamFilters;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, watch};

const BUS_CAPACITY: usize = 16_384;

pub struct VortexHub {
    bus: broadcast::Sender<Arc<VortexTransaction>>,
    filters: watch::Sender<StreamFilters>,
    consumers: Mutex<HashMap<String, BTreeSet<String>>>,
    transactions: AtomicU64,
    last_slot: AtomicU64,
    last_transaction_at: Mutex<Option<DateTime<Utc>>>,
    started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HubStats {
    pub transactions: u64,
    pub last_slot: u64,
    pub last_transaction_at: Option<DateTime<Utc>>,
    pub started_at: DateTime<Utc>,
    pub programs: Vec<String>,
    pub subscribers: usize,
}

impl VortexHub {
    pub fn new(wallet_pubkey: Option<String>) -> Arc<Self> {
        let (bus, _) = broadcast::channel(BUS_CAPACITY);
        let (filters, _) = watch::channel(StreamFilters {
            wallet_pubkey,
            programs: vec![],
        });
        Arc::new(Self {
            bus,
            filters,
            consumers: Mutex::new(HashMap::new()),
            transactions: AtomicU64::new(0),
            last_slot: AtomicU64::new(0),
            last_transaction_at: Mutex::new(None),
            started_at: Utc::now(),
        })
    }

    /// Receiver for the Geyser subscription task.
    pub fn stream_filters(&self) -> watch::Receiver<StreamFilters> {
        self.filters.subscribe()
    }

    pub fn publish(&self, tx: Arc<VortexTransaction>) {
        self.transactions.fetch_add(1, Ordering::Relaxed);
        self.last_slot.fetch_max(tx.slot, Ordering::Relaxed);
        *self.last_transaction_at.lock().unwrap() = Some(tx.received_at);
        // No subscribers is fine; the transaction is simply not consumed.
        let _ = self.bus.send(tx);
    }

    pub fn record_slot(&self, slot: u64) {
        self.last_slot.fetch_max(slot, Ordering::Relaxed);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<VortexTransaction>> {
        self.bus.subscribe()
    }

    /// Replaces the set of programs `consumer` wants streamed.
    pub fn set_programs(&self, consumer: &str, programs: impl IntoIterator<Item = String>) {
        let mut consumers = self.consumers.lock().unwrap();
        consumers.insert(consumer.to_string(), programs.into_iter().collect());
        let union: BTreeSet<String> = consumers.values().flatten().cloned().collect();
        self.filters.send_if_modified(|f| {
            let next: Vec<String> = union.into_iter().collect();
            if f.programs == next {
                return false;
            }
            f.programs = next;
            true
        });
    }

    pub fn stats(&self) -> HubStats {
        HubStats {
            transactions: self.transactions.load(Ordering::Relaxed),
            last_slot: self.last_slot.load(Ordering::Relaxed),
            last_transaction_at: *self.last_transaction_at.lock().unwrap(),
            started_at: self.started_at,
            programs: self.filters.borrow().programs.clone(),
            subscribers: self.bus.receiver_count(),
        }
    }
}

use lazy_static::lazy_static;

use std::{
    collections::{HashMap,BTreeMap},
    sync::{
        atomic::{AtomicU64,Ordering},
        Arc,Mutex
    }
};

/// Key-Value Engine define
pub type KVEngine = BTreeMap<Vec<u8>,Option<Vec<u8>>>;

/// Version define, increment one by one
pub static VERSION:AtomicU64 = AtomicU64::new(1);

// Acquire next version
pub fn acquire_next_version() -> u64 {
    VERSION.fetch_add(1,Ordering::SeqCst)
}
// Active Transaction id
lazy_static!{
    pub static ref ACTIVE_TXN:Arc<Mutex<HashMap<u64, Vec<Vec<u8>>>>> = Arc::new(Mutex::new(HashMap::new()));
}
use std::sync::{    
        Arc,Mutex
    };
use crate::core_opt::core::KVEngine;
use crate::obj::transaction::Transaction;

pub struct MVCC{
    kv:Arc<Mutex<KVEngine>>,
}

impl MVCC{
    pub fn new(kv:KVEngine) -> Self{
        Self{
            kv:Arc::new(Mutex::new(kv)),
        }
    }

    pub fn begin_transaction(&self) -> Transaction {
        Transaction::begin(self.kv.clone())
    }
}
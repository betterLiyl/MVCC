use crate::core_opt::core::*;
use crate::obj::key::*;
use std::{
    collections::{BTreeMap,HashSet},
    sync::{
        Arc,Mutex
    }
};
pub struct Transaction {
    // 底层 KV 存储引擎
    kv: Arc<Mutex<KVEngine>>,
    // 事务版本号
    version: u64,
    // 事务启动时的活跃事务列表
    active_xid: HashSet<u64>,
}

impl Transaction {
    // 开启事务
    pub fn begin(kv: Arc<Mutex<KVEngine>>) -> Self {
        // 获取全局事务版本号
        let version = acquire_next_version();

        let mut active_txn = ACTIVE_TXN.lock().unwrap();
        // 这个 map 的 key 就是当前所有活跃的事务
        let active_xid = active_txn.keys().cloned().collect();

        // 添加到当前活跃事务 id 列表中
        active_txn.insert(version, vec![]);

        // 返回结果
        Self {
            kv,
            version,
            active_xid,
        }
    }

    // 写入数据
    pub fn set(&self, key: &[u8], value: Vec<u8>) {
        self.write(key, Some(value))
    }

    // 删除数据
    pub fn delete(&self, key: &[u8]) {
        self.write(key, None)
    }

    fn write(&self, key: &[u8], value: Option<Vec<u8>>) {
        // 判断当前写入的 key 是否和其他的事务冲突
        // key 是按照 key-version 排序的，所以只需要判断最近的一个 key 即可
        let mut kvengine = self.kv.lock().unwrap();
        for (enc_key, _) in kvengine.iter().rev() {
            let key_version = decode_key(enc_key);
            if key_version.raw_key.eq(key) {
                if !self.is_visible(key_version.version) {
                    panic!("serialization error, try again.");
                }
                break;
            }
        }

        // 写入 TxnWrite
        let mut active_txn = ACTIVE_TXN.lock().unwrap();
        active_txn
            .entry(self.version)
            .and_modify(|keys| keys.push(key.to_vec()))
            .or_insert_with(|| vec![key.to_vec()]);

        // 写入数据
        let enc_key = Key {
            raw_key: key.to_vec(),
            version: self.version,
        };
        kvengine.insert(enc_key.encode(), value);
    }

    // 读取数据，从最后一条数据进行遍历，找到第一条可见的数据
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let kvengine = self.kv.lock().unwrap();
        for (k, v) in kvengine.iter().rev() {
            let key_version = decode_key(k);
            if key_version.raw_key.eq(key) && self.is_visible(key_version.version) {
                return v.clone();
            }
        }
        None
    }

    // 打印出所有可见的数据
    pub fn print_all(&self) {
        let mut records = BTreeMap::new();
        let kvengine = self.kv.lock().unwrap();
        for (k, v) in kvengine.iter() {
            let key_version = decode_key(k);
            if self.is_visible(key_version.version) {
                records.insert(key_version.raw_key.to_vec(), v.clone());
            }
        }

        for (k, v) in records.iter() {
            if let Some(value) = v {
                print!(
                    "{}={} ",
                    String::from_utf8_lossy(k),
                    String::from_utf8_lossy(value)
                );
            }
        }
        println!("");
    }

    // 提交事务
    pub fn commit(&self) {
        // 清除活跃事务列表中的数据
        let mut active_txn = ACTIVE_TXN.lock().unwrap();
        active_txn.remove(&self.version);
    }

    // 回滚事务
    pub fn rollback(&self) {
        // 清除写入的数据
        let mut active_txn = ACTIVE_TXN.lock().unwrap();
        if let Some(keys) = active_txn.get(&self.version) {
            let mut kvengine = self.kv.lock().unwrap();
            for k in keys {
                let enc_key = Key {
                    raw_key: k.to_vec(),
                    version: self.version,
                };
                let res = kvengine.remove(&enc_key.encode());
                assert!(res.is_some());
            }
        }

        // 清除活跃事务列表中的数据
        active_txn.remove(&self.version);
    }

    // 判断一个版本的数据对当前事务是否可见
    // 1. 如果是另一个活跃事务的修改，则不可见
    // 2. 如果版本号比当前大，则不可见
    fn is_visible(&self, version: u64) -> bool {
        if self.active_xid.contains(&version) {
            return false;
        }
        version <= self.version
    }
}
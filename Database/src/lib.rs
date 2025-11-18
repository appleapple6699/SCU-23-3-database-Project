use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod storage;
mod txn;
mod index;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Config {
    pub data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self { data_dir: PathBuf::from("data") }
    }
}

struct Inner {
    map: HashMap<Vec<u8>, Vec<u8>>, 
    wal: storage::wal::Wal,
}

#[derive(Clone)]
pub struct Database {
    inner: Arc<Mutex<Inner>>, 
}

impl Database {
    pub fn open(config: Config) -> Result<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        let wal_path = config.data_dir.join("wal.log");
        let mut wal = storage::wal::Wal::open(&wal_path)?;
        let mut map = HashMap::new();
        wal.replay(&mut map)?;
        Ok(Self { inner: Arc::new(Mutex::new(Inner { map, wal })) })
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        g.wal.append_put(key, value)?;
        g.map.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let g = self.inner.lock().unwrap();
        Ok(g.map.get(key).cloned())
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        g.wal.append_delete(key)?;
        g.map.remove(key);
        Ok(())
    }

    pub fn begin_txn(&self, _opts: txn::TxnOptions) -> txn::Txn {
        txn::Txn::new(self.inner.clone())
    }
}

pub mod prelude {
    pub use crate::{Config, Database, Result};
}
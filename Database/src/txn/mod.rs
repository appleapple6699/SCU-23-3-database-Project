use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::Result;

pub struct TxnOptions {
    read_write: bool,
}

impl TxnOptions {
    pub fn read_write() -> Self { Self { read_write: true } }
    pub fn read_only() -> Self { Self { read_write: false } }
}

enum Op {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

struct InnerState {
    map: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct Txn {
    inner: Arc<Mutex<crate::Inner>>, 
    staged: Vec<Op>,
    overlay: HashMap<Vec<u8>, Option<Vec<u8>>>,
    opts: TxnOptions,
}

impl Txn {
    pub fn new(inner: Arc<Mutex<crate::Inner>>) -> Self {
        Self { inner, staged: Vec::new(), overlay: HashMap::new(), opts: TxnOptions::read_write() }
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.staged.push(Op::Put(key.to_vec(), value.to_vec()));
        self.overlay.insert(key.to_vec(), Some(value.to_vec()));
        Ok(())
    }

    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.staged.push(Op::Delete(key.to_vec()));
        self.overlay.insert(key.to_vec(), None);
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if let Some(v) = self.overlay.get(key) { return Ok(v.clone()); }
        let g = self.inner.lock().unwrap();
        Ok(g.map.get(key).cloned())
    }

    pub fn commit(mut self) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        for op in self.staged.drain(..) {
            match op {
                Op::Put(k, v) => {
                    g.wal.append_put(&k, &v)?;
                    g.map.insert(k, v);
                }
                Op::Delete(k) => {
                    g.wal.append_delete(&k)?;
                    g.map.remove(&k);
                }
            }
        }
        Ok(())
    }

    pub fn rollback(self) -> Result<()> { Ok(()) }
}